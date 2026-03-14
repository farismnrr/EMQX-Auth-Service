use crate::dtos::response_dto::ErrorResponseDTO;
use actix_web::{
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    Error, HttpResponse,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use log::debug;
use std::env;

#[derive(Clone)]
pub struct ApiKeyMiddleware;

impl<S, B> Transform<S, ServiceRequest> for ApiKeyMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiKeyMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let api_key = env::var("API_KEY").unwrap_or_default();
        ok(ApiKeyMiddlewareService { service, api_key })
    }
}

#[derive(Clone)]
pub struct ApiKeyMiddlewareService<S> {
    service: S,
    api_key: String,
}

impl<S, B> Service<ServiceRequest> for ApiKeyMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let expected_key = self.api_key.clone();
        let path = req.path().to_string();
        let has_auth_header = req.headers().get("x-api-key").is_some();
        let auth_value = req
            .headers()
            .get("x-api-key")
            .and_then(|v| v.to_str().ok())
            .map(str::trim)
            .unwrap_or("");
        debug!(
            "[Middleware | ApiKey] call - path='{}' auth_header_present={} auth_len={} expected_key_set={}",
            path,
            has_auth_header,
            auth_value.len(),
            !expected_key.is_empty()
        );

        let is_valid = !expected_key.is_empty() && is_authorized(auth_value, &expected_key);
        if !is_valid {
            debug!("[Middleware | ApiKey] Unauthorized request to '{}'", path);
            let res = HttpResponse::Unauthorized()
                .insert_header((header::CONTENT_TYPE, "application/json"))
                .json(ErrorResponseDTO {
                    success: false,
                    message: "Unauthorized",
                    details: None::<()>,
                    result: None,
                });
            return Box::pin(async move { Ok(req.into_response(res.map_into_right_body())) });
        }

        debug!("[Middleware | ApiKey] Authorized request to '{}'", path);
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.map_into_left_body())
        })
    }
}

fn is_authorized(header_value: &str, expected_key: &str) -> bool {
    header_value == expected_key
}
