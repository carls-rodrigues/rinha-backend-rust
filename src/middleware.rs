pub async fn middleware() -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().finish())
}
