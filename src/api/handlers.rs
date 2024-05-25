use axum::http::StatusCode;
use axum::response::*;
use axum::routing::*;
use axum::extract::*;
use axum::http;

use crate::app_state::AppState;
use crate::api::error::*;

pub fn handlers(state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/api/v1/users", post(register_new_user))
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
    email: String,
    public_key: String,
    roles: Option<Vec<String>>
}

#[axum::debug_handler]
async fn register_new_user(
    Extension(ClientId(client_id)): Extension<ClientId>,
    State(AppState { db_pool, .. }): State<AppState>,
    Query(UserRegisterRequest {
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
}