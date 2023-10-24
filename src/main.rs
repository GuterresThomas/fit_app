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
struct Personal {
    personal_id: i32,
    nome: String,
    cpf: String,
    telefone: String,
    email: String,
    idade: i32,
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
#[derive(serde::Deserialize, serde::Serialize)]
struct AlunoPersonal {
    aluno_id: i32,
    personal_id: i32,
    nome_aluno: String,
    email_aluno: String,
    telefone_aluno: String,
    cpf_aluno: String,
    nome_personal: String,
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

    

    let get_all_alunos = warp::path!("alunos")
    .and(warp::get())
    .and(db.clone())
    .and_then(|client: Arc<Client>| async move {
        // Consulta SQL para buscar todos os alunos
        let query = "SELECT * FROM alunos";

        match client.query(query, &[]).await {
            Ok(rows) => {
                let mut alunos = Vec::new();

                for row in rows {
                    let aluno = Aluno {
                        aluno_id: row.get(0),
                        personal_id: row.get(1),
                        nome: row.get(2),
                        email: row.get(3),
                        telefone: row.get(4),
                        cpf: row.get(5),
                    };
                    alunos.push(aluno);
                }

                Ok(warp::reply::json(&alunos))
            }
            Err(err) => {
                let error_message = format!("Erro na consulta de alunos: {}", err);
                Err(custom(CustomError(error_message)))
            }
        }
    });

    let get_alunos_de_personal = warp::path!("alunos" / "personal" / i32)
    .and(warp::get())
    .and(db.clone())
    .and_then(|personal_id: i32, client: Arc<Client>| async move {
        // Consulta SQL para buscar todos os alunos associados a um personal e o nome do personal
        let query = "SELECT alunos.*, users.nome AS nome_personal FROM alunos JOIN users ON alunos.personal_id = users.user_id WHERE alunos.personal_id = $1";

        match client.query(query, &[&personal_id]).await {
            Ok(rows) => {
                let mut alunos = Vec::new();

                for row in rows {
                    let aluno = Aluno {
                        aluno_id: row.get(0),
                        personal_id: row.get(1),
                        nome: row.get(2),
                        email: row.get(3),
                        telefone: row.get(4),
                        cpf: row.get(5),
                    };
                    let nome_personal: String = row.get(6);

                    let aluno_com_nome_personal = AlunoPersonal {
                        aluno_id: aluno.aluno_id,
                        personal_id: aluno.personal_id,
                        nome_aluno: aluno.nome,
                        email_aluno: aluno.email,
                        telefone_aluno: aluno.telefone,
                        cpf_aluno: aluno.cpf,
                        nome_personal,
                    };

                    alunos.push(aluno_com_nome_personal);
                }

                Ok(warp::reply::json(&alunos))
            }
            Err(err) => {
                let error_message = format!("Erro na consulta de alunos do personal trainer: {}", err);
                Err(custom(CustomError(error_message)))
            }
        }
    });


    let get_personais = warp::get()
    .and(warp::path!("todos_personais"))
    .and(db.clone())
    .and_then(|client: Arc<Client>| async move {
        let query = format!("SELECT user_id, nome, cpf, telefone, email, idade FROM users");

        match client.query(&query, &[]).await {              
               
                Ok(rows) => {
                    let personais: Vec<Personal> = rows 
                    .into_iter()
                    .map(|row| Personal {
                        personal_id: row.get("user_id"),
                        nome: row.get("nome"),
                        cpf: row.get("cpf"),
                        telefone: row.get("telefone"),
                        email: row.get("email"),
                        idade: row.get("idade"),
                    })
                    .collect();
                
                Ok(warp::reply::json(&personais))
                }
                Err(err) => {
                    let error_message = format!("Erro na consulta de personal trainer: {}", err);
                    Err(custom(CustomError(error_message)))
                }
            }
    });

    let get_alunos_treinos = warp::path!("alunos"/ "treinos"/ i32)
    .and(warp::get())
    .and(db.clone())
    .and_then(|aluno_id: i32, client: Arc<Client>| async move {
        let query = format!("SELECT
        alunos.aluno_id,
        alunos.personal_id,
        alunos.nome AS nome_aluno,
        alunos.email AS email_aluno,
        alunos.telefone AS telefone_aluno,
        alunos.cpf AS cpf_aluno,
        treinos.treino_id,
        treinos.data_do_treino,
        treinos.descricao_do_treino,
        users.nome AS nome_personal
    FROM
        alunos
    JOIN
        users ON alunos.personal_id = users.user_id
    JOIN
        treinos ON alunos.aluno_id = treinos.aluno_id
    WHERE
     alunos.aluno_id = $1;");
     match client.query(&query, &[&aluno_id]).await {
        Ok(rows) => {
            let mut result = Vec::new();
            for row in rows {
                let aluno = AlunoPersonal {
                    aluno_id: row.get("aluno_id"),
                    personal_id: row.get("personal_id"),
                    nome_aluno: row.get("nome_aluno"),
                    email_aluno: row.get("email_aluno"),
                    telefone_aluno: row.get("telefone_aluno"),
                    cpf_aluno: row.get("cpf_aluno"),
                    nome_personal: row.get("nome_personal"),
                };
                let treino = Treino {
                    treino_id: row.get("treino_id"),
                    aluno_id: row.get("aluno_id"),
                    data_do_treino: row.get("data_do_treino"),
                    descricao_do_treino: row.get("descricao_do_treino"),
                };
                result.push((aluno, treino));
            }
            Ok(warp::reply::json(&result))
        }
        Err(err) => {
            let error_message = format!("Erro na consulta de alunos e treinos: {}", err);
            Err(custom(CustomError(error_message)))
        }
    }
    });

    let routes = login.
    or(create_user).
    or(create_treino).
    or(create_aluno).
    or(get_all_alunos).
    or(get_alunos_de_personal).
    or(get_personais).
    or(get_alunos_treinos)
    .with(cors);
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;

    Ok(())
}
