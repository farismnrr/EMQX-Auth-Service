use actix_web::{
    body::{EitherBody, MessageBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct RateLimiter {
    requests_per_minute: u32,
    storage: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddleware {
            service,
            requests_per_minute: self.requests_per_minute,
            storage: Arc::clone(&self.storage),
        }))
    }
}

pub struct RateLimitMiddleware<S> {
    service: S,
    requests_per_minute: u32,
    storage: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();

        let now = Instant::now();
        let one_minute_ago = now - Duration::from_secs(60);

        {
            let mut storage = self.storage.lock().unwrap();
            let timestamps = storage.entry(ip.clone()).or_insert_with(Vec::new);

            // Remove timestamps older than 1 minute
            timestamps.retain(|&t| t > one_minute_ago);

            if timestamps.len() >= self.requests_per_minute as usize {
                let response = HttpResponse::TooManyRequests()
                    .content_type("application/json")
                    .body("{\"success\":false,\"message\":\"Too many requests\",\"code\":\"TOO_MANY_REQUESTS\"}");
                
                let (request, _pl) = req.into_parts();
                return Box::pin(ready(Ok(ServiceResponse::new(
                    request,
                    response.map_into_right_body(),
                ))));
            }

            timestamps.push(now);
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.map_into_left_body())
        })
    }
}
