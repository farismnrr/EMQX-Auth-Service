//! API Key Authentication Middleware

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse, body::EitherBody,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

/// Middleware for API Key authentication
pub struct ApiKeyMiddleware {
    api_key: String,
}

impl ApiKeyMiddleware {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ApiKeyMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiKeyMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiKeyMiddlewareService {
            service,
            api_key: self.api_key.clone(),
        }))
    }
}

pub struct ApiKeyMiddlewareService<S> {
    service: S,
    api_key: String,
}

impl<S, B> Service<ServiceRequest> for ApiKeyMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let mut authenticated = false;

        // Check x-api-key header
        if let Some(key) = req.headers().get("x-api-key") {
            if let Ok(key_str) = key.to_str() {
                if key_str == self.api_key {
                    authenticated = true;
                }
            }
        }

        // Check Authorization header (used by EMQX setup and some clients)
        if !authenticated {
            if let Some(auth) = req.headers().get("Authorization") {
                if let Ok(auth_str) = auth.to_str() {
                    // Support both plain key and "Bearer <key>"
                    if auth_str == self.api_key || auth_str.strip_prefix("Bearer ").is_some_and(|s| s == self.api_key) {
                        authenticated = true;
                    }
                }
            }
        }

        if authenticated {
            let fut = self.service.call(req);
            Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_left_body())
            })
        } else {
            let (request, _pl) = req.into_parts();
            let res = HttpResponse::Unauthorized()
                .json(serde_json::json!({
                    "success": false,
                    "message": "Unauthorized: Invalid or missing API Key"
                }));
            
            Box::pin(ready(Ok(ServiceResponse::new(request, res.map_into_right_body()))))
        }
    }
}
