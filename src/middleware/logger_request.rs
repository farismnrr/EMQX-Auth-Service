use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use futures_util::future::LocalBoxFuture;
use log::info;
use std::rc::Rc;
use std::task::{Context, Poll};
use std::time::Instant;

pub struct RequestLoggerMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestLoggerMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestLoggerMiddlewareImpl<S>;
    type Future = LocalBoxFuture<'static, Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        Box::pin(async move {
            Ok(RequestLoggerMiddlewareImpl {
                service: Rc::new(service),
            })
        })
    }
}

pub struct RequestLoggerMiddlewareImpl<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequestLoggerMiddlewareImpl<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = Rc::clone(&self.service);
        let method = req.method().clone();
        let path = req.path().to_string();
        let peer_ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();
        let start = Instant::now();

        Box::pin(async move {
            let res = svc.call(req).await?;
            let duration = start.elapsed();
            let status = res.response().status().as_u16();

            info!(
                "[{}] {} {} | {} | {:?}",
                status, method, path, peer_ip, duration
            );

            Ok(res)
        })
    }
}
