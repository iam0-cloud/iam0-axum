use axum::http::StatusCode;
use axum::response::*;
use axum::routing::*;
use axum::extract::*;
use axum::http;
use anyhow::Context;
use p256::elliptic_curve::sec1::FromEncodedPoint;

use crate::app_state::AppState;
use crate::api::error::*;

pub fn handlers(state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/api/v1/register", post(register_new_user))
        .route("/api/v1/login", post(user_login))
        .layer(axum::middleware::from_fn_with_state(state.clone(), api_auth)) 
        .with_state(state)
}

#[derive(Clone, Copy)]
struct ClientId(i64);

async fn api_auth(
    State(AppState { db_pool, .. }): State<AppState>,
    mut req: Request,
    next: axum::middleware::Next
) -> Result<Response, http::StatusCode> {
    let auth_header =  req.headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let api_key = if let Some(auth_header) = auth_header {
        if !auth_header.starts_with("Bearer ") {
            return Err(StatusCode::BAD_REQUEST);
        } else {
            if let Some(api_key) = auth_header.split_ascii_whitespace().last() {
                api_key
            } else {
                return Err(StatusCode::BAD_REQUEST)
            }
        }
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let Ok(client_id_query) = sqlx::query!(
        r#"
            select client_id from clients where api_key = ?
        "#,
        api_key
    ).fetch_one(&db_pool).await else {
        return Err(StatusCode::UNAUTHORIZED)
    };

    // SAFETY: the unwrap is guaranteed to succeed
    req.extensions_mut()
        .insert(ClientId(client_id_query.client_id.unwrap()));

    Ok(next.run(req).await)

}

#[derive(serde::Deserialize)]
struct UserRegisterRequest {
    username: String,
    email: String,
    public_key: String,
    roles: Option<Vec<String>>
}

async fn register_new_user(
    Extension(ClientId(client_id)): Extension<ClientId>,
    State(AppState { db_pool, .. }): State<AppState>,
    Query(UserRegisterRequest {
        username,
        email,
        public_key,
        roles
    }): Query<UserRegisterRequest>
) -> Result<Response, Error> {
    let record = sqlx::query!(r#"
        insert into users (client_id, email, public_key)
        values (?, ?, ?)
        returning user_id
    "#, client_id, email, public_key)
        .fetch_one(&db_pool).await?;
    let user_id = record.user_id;

    if let Some(roles) = roles {
        for role in &roles {
            let record = sqlx::query!(
                r#"
                    select role_id from roles where client_id = ? and role_name = ?
                "#,
                client_id,
                role
            )
                .fetch_one(&db_pool).await.map_err(|_| ApiError::UnknownRole)?;

            sqlx::query!(r#"
                insert into user_roles_rel (user_id, role_id)
                values (?, ?)
            "#, user_id, record.role_id)
                .execute(&db_pool).await?;
        }
    }
 
    Ok((
        http::StatusCode::OK,
        Json(serde_json::json!({
            "user_id": user_id
        }))
    ).into_response())
}

#[derive(serde::Deserialize)]
struct UserLoginRequest {
    email: Option<String>,
    username: Option<String>,
    proof: p256::Scalar,
    commitment: p256::AffinePoint,

    /// Signed username/email
    payload: String,
}

async fn user_login(
    Extension(ClientId(client_id)): Extension<ClientId>,
    State(AppState { db_pool, .. }): State<AppState>,
    Query(UserLoginRequest { 
        email,
        username,
        commitment,
        proof,
        payload 
    }): Query<UserLoginRequest>
) -> Result<Response, Error> {
    let public_key = if let Some(email) = email {
        let record = sqlx::query!( 
            r#"
                select public_key
                from users
                where email = ? and client_id = ?
            "#,
            email, client_id
        )
            .fetch_one(&db_pool).await
            .context("failed to query public key for this email")?;

        let encoded_point = p256::EncodedPoint::from_bytes(record.public_key)
            .context("failed to decode public key from database")?;
        p256::AffinePoint::from_encoded_point(&encoded_point).unwrap()
    } else if let Some(username) = username {
        let record = sqlx::query!(
            r#"
                select public_key
                from users
                where username = ? and client_id = ?
            "#,
            username, client_id
        )
            .fetch_one(&db_pool).await
            .context("failed to query public key for this username")?;

        let encoded_point = p256::EncodedPoint::from_bytes(record.public_key)
            .context("failed to decode public key from database")?;
        p256::AffinePoint::from_encoded_point(&encoded_point).unwrap()
    } else {
        return Err(anyhow::anyhow!("missing email or username").into());
    };

    // TODO: zkp verify
    let verify = true;
    // NistP256.verify(&payload, &public_key, &proof, &commitment);

    if verify == true {
        // TODO
        todo!("generate token with our custom token format")
    } else {
        Ok((
            StatusCode::UNAUTHORIZED,
            "proof failed or user doesn't exist"
        ).into_response())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;

    async fn build_app() -> axum::Router {
        let app_state = crate::create_server(true).await.unwrap();
        handlers(app_state)
    }

    #[tokio::test]
    async fn register_new_user_simple() -> anyhow::Result<()> {
        let req = Request::builder()
            .method(http::Method::POST)
            .uri("/api/v1/users?email=user3@email.com&public_key=zzz")
            .header(http::header::AUTHORIZATION, "Bearer super_secret")
            .body(axum::body::Body::empty())?;
        
        let res = build_app().await.oneshot(req).await?;
        let (parts, _) = res.into_parts();

        assert_eq!(
            parts.status,
            http::StatusCode::OK
        );

        Ok(())
    }

    #[tokio::test]
    async fn register_new_user_unknown_client() -> anyhow::Result<()> {
        let req = Request::builder()
            .method(http::Method::POST)
            .uri("/api/v1/users?email=user3@email.com&public_key=zzz")
            .header(http::header::AUTHORIZATION, "Bearer invalid_secret")
            .body(axum::body::Body::empty())?;
    
        let res = build_app().await.oneshot(req).await?;
        let (parts, _) = res.into_parts();

        assert_eq!(
            parts.status,
            http::StatusCode::UNAUTHORIZED
        );

        Ok(())
    }

    #[tokio::test]
    async fn missing_api_key() -> anyhow::Result<()> {
        // Missing token
        let req = Request::builder()
            .method(http::Method::POST)
            .uri("/api/v1/users?email=user3@email.com&public_key=zzz")
            .body(axum::body::Body::empty())?;

        let res = build_app().await.oneshot(req).await?;
        let (parts, _) = res.into_parts();

        assert_eq!(
            parts.status,
            http::StatusCode::UNAUTHORIZED
        );

        Ok(())
    }

    #[tokio::test]
    async fn invalid_api_key() -> anyhow::Result<()> {
        // Missing bearer
        let req = Request::builder()
            .method(http::Method::POST)
            .uri("/api/v1/users?email=user3@email.com&public_key=zzz")
            .header(http::header::AUTHORIZATION, "super_secret")
            .body(axum::body::Body::empty())?;

        let res = build_app().await.oneshot(req).await?;
        let (parts, _) = res.into_parts();

        assert_eq!(
            parts.status,
            http::StatusCode::BAD_REQUEST
        );

        Ok(())
    }
}