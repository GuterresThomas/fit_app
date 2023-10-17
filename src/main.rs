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
    telefone: String,
    email: String,
    idade: i32,
    user_type: String,
    senha: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Aluno {
    aluno_id: i32,
    personal_id: i32, // Referência ao personal
    nome: String,
    email: String,
    telefone: String,
    cpf: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Treino {
    treino_id: i32,
    aluno_id: i32,
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
            let insert_query = format!("INSERT INTO users (user_id, nome, cpf, telefone, email, idade, user_type, senha) VALUES ('{}','{}','{}','{}','{}','{}','{}', '{}')", user.user_id, user.nome, user.cpf, user.telefone, user.email, user.idade, user.user_type, user.senha);
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

    let create_treino = warp::post()
    .and(warp::path("treino_create"))
    .and(warp::body::json())
    .and(db.clone())
    .and_then(|treino: Treino, client: Arc<Client>| async move {
        // Verifique se o usuário com o ID especificado existe
        let aluno_id = treino.aluno_id;
        let user_query = "SELECT aluno_id FROM alunos WHERE aluno_id = $1";

        match client.query(user_query, &[&aluno_id]).await {
            Ok(user_rows) => {
                if user_rows.is_empty() {
                    // O usuário com o ID especificado não foi encontrado
                    let error_message = "Usuário não encontrado".to_string();
                    return Err(custom(CustomError(error_message)));
                }

                // O usuário existe, então insira o treino associado
                let insert_query = format!("INSERT INTO treinos (treino_id, aluno_id, data_do_treino, descricao_do_treino) VALUES ('{}','{}','{}','{}')", treino.treino_id, treino.aluno_id, treino.data_do_treino, treino.descricao_do_treino);
                match client.execute(&insert_query, &[]).await {
                    Ok(rows) if rows == 1 => {
                        Ok(warp::reply::json(&treino))
                    }
                    _ => {
                        let error_message = "Falha ao adicionar treino".to_string();
                        Err(custom(CustomError(error_message)))
                    }
                }
            }
            Err(err) => {
                let error_message = format!("Erro durante a verificação do usuário: {}", err);
                Err(custom(CustomError(error_message)))
            }
        }
    });

    let create_aluno = warp::post()
    .and(warp::path("aluno_create"))
    .and(warp::body::json())
    .and(db.clone())
    .and_then(|aluno: Aluno, client: Arc<Client>| async move {
        // Verifique se o usuário com o ID especificado existe
        let user_id = aluno.personal_id;
        let user_query = "SELECT user_id FROM users WHERE user_id = $1";

        match client.query(user_query, &[&user_id]).await {
            Ok(user_rows) => {
                if user_rows.is_empty() {
                    // O personal trainer com o ID especificado não foi encontrado
                    let error_message = "Personal Trainer não encontrado".to_string();
                    return Err(custom(CustomError(error_message)));
                }

                // O personal trainer existe, então insira o aluno associado
                let insert_query = format!("INSERT INTO alunos (aluno_id, personal_id, nome, email, telefone, cpf) VALUES ('{}','{}','{}','{}','{}','{}')", aluno.aluno_id, aluno.personal_id, aluno.nome, aluno.email, aluno.telefone, aluno.cpf);
                match client.execute(&insert_query, &[]).await {
                    Ok(rows) if rows == 1 => {
                        Ok(warp::reply::json(&aluno))
                    }
                    _ => {
                        let error_message = "Falha ao adicionar aluno".to_string();
                        Err(custom(CustomError(error_message)))
                    }
                }
            }
            Err(err) => {
                let error_message = format!("Erro durante a verificação do personal trainer: {}", err);
                Err(custom(CustomError(error_message)))
            }
        }
    });




    let routes = login.or(create_user).or(create_treino).or(create_aluno).with(cors);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;

    Ok(())
}
