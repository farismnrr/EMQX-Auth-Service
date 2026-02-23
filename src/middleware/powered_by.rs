use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
};
use futures_util::future::{LocalBoxFuture, Ready, ok};
use log::debug;

#[derive(Clone)]
pub struct PoweredByMiddleware;

impl<S, B> Transform<S, ServiceRequest> for PoweredByMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = PoweredByMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(PoweredByMiddlewareService { service })
    }
}

#[derive(Clone)]
pub struct PoweredByMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for PoweredByMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();
        let fut = self.service.call(req);
        Box::pin(async move {
            let mut res = fut.await?;
            res.headers_mut().insert(
                header::HeaderName::from_static("x-powered-by"),
                header::HeaderValue::from_static("IoTNet"),
            );
            debug!(
                "[Middleware | PoweredBy] set 'x-powered-by' header for path='{}'",
                path
            );
            Ok(res)
        })
    }
}
