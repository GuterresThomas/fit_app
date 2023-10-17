use warp::Filter;
use tokio_postgres::{NoTls, Error, Client};
use std::sync::Arc;
use warp::reject::custom;
use serde::{Deserialize, Serialize};

// Define um tipo de erro personalizado que implementa Reject
#[derive(Debug)]
struct CustomError(String);

impl warp::reject::Reject for CustomError {}

// Define estruturas de dados para usuário, aluno e personal trainer
#[derive(serde::Deserialize, serde::Serialize)]
struct User {
    user_id: i32,
    nome: String,
    cpf: String,
    telefone: Option<String>,
    email: String,
    idade: i32,
    user_type: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Student {
    id: i32,
    user_id: i32, // Referência ao usuário
    treinos: Vec<Treino>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Treino {
    treino_id: i32,
    data_do_treino: String,
    descricao_do_treino: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    senha: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    user_id: i32,
    nome: String,
    // Outras informações do usuário que você deseja retornar após o login
}


#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) =
    tokio_postgres::connect("host=localhost user=postgres password=1234 dbname=postgres", NoTls)
        .await?;
tokio::spawn(connection);
    let client = Arc::new(client);

    let db = warp::any().map(move || client.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "DELETE", "PUT"])
        .allow_headers(vec!["Content-Type"])
        .max_age(3600);

        let login = warp::post()
        .and(warp::path("login"))
        .and(warp::body::json())
        .and(db.clone())
        .and_then(|login_request: LoginRequest, client: Arc<Client>| async move {
            // Realize a consulta no banco de dados para verificar se o usuário com as credenciais fornecidas existe
            let query = "SELECT user_id, nome FROM users WHERE email = $1 AND senha = $2";

            match client.query(query, &[&login_request.email, &login_request.senha]).await {
                Ok(rows) => {
                    if let Some(row) = rows.iter().next() {
                        // O login foi bem-sucedido, retornando informações do usuário
                        let user_id: i32 = row.get(0);
                        let nome: String = row.get(1);

                        let login_response = LoginResponse { user_id, nome };

                        Ok(warp::reply::json(&login_response))
                    } else {
                        // Credenciais inválidas
                        let error_message = "Credenciais inválidas".to_string();
                        Err(custom(CustomError(error_message)))
                    }
                }
                Err(err) => {
                    // Ocorreu um erro na consulta
                    let error_message = format!("Erro durante o login: {}", err);
                    Err(custom(CustomError(error_message)))
                }
            }
        });

        let create_user = warp::post()
        .and(warp::path("user_create"))
        .and(warp::body::json())
        .and(db.clone())
        .and_then(|user: User, client: Arc<Client>| async move {
            let insert_query = format!("INSERT INTO users (user_id, nome, cpf, telefone, email, idade, user_type)");
            match client.execute(&insert_query, &[]).await {
                Ok(rows) if rows == 1 => {
                    Ok(warp::reply::json(&user))
                }
            _ => {
                let error_message = "Falha ao adicionar usuário".to_string();
                Err(custom(CustomError(error_message)))
            },
        }

        
    });

    


    let routes = login.or(create_user).with(cors);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;

    Ok(())
}
