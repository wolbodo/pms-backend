#![feature(custom_derive, plugin)]
#![plugin(rocket_codegen)]
#![plugin(postgres_macros)]
extern crate rocket;
extern crate rocket_contrib;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;

#[macro_use]
extern crate lazy_static;

extern crate r2d2;
extern crate r2d2_postgres;

use std::str::FromStr;

use serde_json::Value;
use r2d2_postgres::{TlsMode, PostgresConnectionManager};
use r2d2_postgres::postgres::error::Error as PgError;

use rocket::Outcome;
use rocket::http::Status;
use rocket::request::{self, Request, FromRequest};
use rocket::response::status;
use rocket_contrib::JSON;

lazy_static! {
    pub static ref DB_POOL: r2d2::Pool<r2d2_postgres::PostgresConnectionManager> = setup_db();
}

macro_rules! db_call {
  ($query:expr, $args:expr, $error:expr) => ({
    let db = connection();
    let stmt = db.prepare(
      sql!($query)
    ).unwrap();

    let rows = match stmt.query($args) {
        Ok(rows) => rows,
        Err(PgError::Db(err)) => return Err(status::Custom(Status::BadRequest, err.message)),
        Err(err) => return Err(status::Custom(Status::InternalServerError, err.to_string())),
    };

    let object: Value = match rows.get(0).get(0) {
        Some(value) => value,
        None => return Err($error)
    };
    Ok(JSON(object))
  })
}

fn setup_db() -> r2d2::Pool<r2d2_postgres::PostgresConnectionManager> {
    let config = r2d2::Config::default();
    let manager = PostgresConnectionManager::new("postgres://pms@%2Frun%2Fpostgresql", TlsMode::None).unwrap();
    r2d2::Pool::new(config, manager).unwrap()
}

fn connection() -> r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager> {
    let pool = DB_POOL.clone();
    pool.get().unwrap()
}

struct APIContext {
  token: String
}

impl<'a, 'r> FromRequest<'a, 'r> for APIContext {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<APIContext, ()> {
        let keys: Vec<_> = request.headers().get("Authorization").collect();
        if keys.len() != 1 {
            return Outcome::Failure((Status::Unauthorized, ()));
        }

        let key = keys[0];
        if key == "test" {
            return Outcome::Forward(());
        }

        return Outcome::Success(APIContext {token: key.to_string()});
    }
}



// Methods

#[derive(Serialize, Deserialize)]
struct Login {
    user: String,
    password: String
}

#[post("/login", data = "<login>")]
fn login(login: JSON<Login>) -> Result<JSON<Value>, status::Custom<String>> {
  db_call!(
    "SELECT login(emailaddress := $1, password := $2);",
    &[&login.0.user, &login.0.password],
    status::Custom(Status::NotFound, "Id not found (or no read access)".to_string())
  )
}

#[post("/people", data = "<person>")]
fn people_add(api: APIContext, person: JSON<Value>) -> Result<JSON<Value>, status::Custom<String>> {
  db_call!(
    "SELECT people_add(token := $1, data := $2);",
    &[&api.token, &person.0],
    status::Custom(Status::NotFound, "Id not found (or no read access)".to_string())
  )
}

#[get("/people")]
fn people_get(api: APIContext) -> Result<JSON<Value>, status::Custom<String>> {
  db_call!(
    "SELECT people_get(token := $1);",
    &[&api.token],
    status::Custom(Status::NotFound, "Id not found (or no read access)".to_string())
  )
}

#[get("/people/<id>")]
fn people_get_i(api: APIContext, id: i32) -> Result<JSON<Value>, status::Custom<String>> {
  db_call!(
    "SELECT people_get(token := $1, people_id := $2);",
    &[&api.token, &id],
    status::Custom(Status::NotFound, "Id not found (or no read access)".to_string())
  )
}

#[put("/people/<id>", data = "<person>")]
fn people_set_i(api: APIContext, id: i32, person: JSON<Value>) -> Result<JSON<Value>, status::Custom<String>> {
  db_call!(
    "SELECT people_set(token := $1, people_id := $2, data := $3);",
    &[&api.token, &id, &person.0],
    status::Custom(Status::NotFound, "Id not found (or no read access)".to_string())
  )
}

#[post("/roles", data = "<role>")]
fn roles_add(api: APIContext, role: JSON<Value>) -> Result<JSON<Value>, status::Custom<String>> {
  db_call!(
    "SELECT roles_add(token := $1, data := $2);",
    &[&api.token, &role.0],
    status::Custom(Status::NotFound, "Id not found (or no read access)".to_string())
  )
}

#[get("/roles")]
fn roles_get(api: APIContext) -> Result<JSON<Value>, status::Custom<String>> {
  db_call!(
    "SELECT roles_get(token := $1);",
    &[&api.token],
    status::Custom(Status::NotFound, "Id not found (or no read access)".to_string())
  )
}

#[get("/roles/<id>")]
fn roles_get_i(api: APIContext, id: i32) -> Result<JSON<Value>, status::Custom<String>> {
  db_call!(
    "SELECT roles_get(token := $1, roles_id := $2);",
    &[&api.token, &id],
    status::Custom(Status::NotFound, "Id not found (or no read access)".to_string())
  )
}

#[put("/roles/<id>", data = "<role>")]
fn roles_set(api: APIContext, id: i32, role: JSON<Value>) -> Result<JSON<Value>, status::Custom<String>> {
  db_call!(
    "SELECT roles_set(token := $1, roles_id := $2, data := $3);",
    &[&api.token, &id, &role.0],
    status::Custom(Status::NotFound, "Id not found (or no read access)".to_string())
  )
}

#[get("/permissions")]
fn permissions_get(api: APIContext) -> Result<JSON<Value>, status::Custom<String>> {
  db_call!(
    "SELECT roles_permissions_get(token := $1);",
    &[&api.token],
    status::Custom(Status::NotFound, "Id not found (or no read access)".to_string())
  )
}

#[get("/fields")]
fn fields_get(api: APIContext) -> Result<JSON<Value>, status::Custom<String>> {
  db_call!(
    "SELECT fields_get(token := $1);",
    &[&api.token],
    status::Custom(Status::NotFound, "Id not found (or no read access)".to_string())
  )
}

#[get("/fields/<table>")]
fn fields_get_i(api: APIContext, table: String) -> Result<JSON<Value>, status::Custom<String>> {
  db_call!(
    "SELECT fields_get(token := $1, ref_table := $2);",
    &[&api.token, &table],
    status::Custom(Status::NotFound, "Id not found (or no read access)".to_string())
  )
}

#[put("/fields")]
fn fields_set() -> Result<JSON<Value>, status::Custom<String>> {
  Ok(JSON(Value::Bool(false)))
  // db_call!(
    // "SELECT fields_set(token := $1, people_id := $2, data := $3);",
    // &[&api.token, &id, &person.0],
    // status::Custom(Status::InternalServerError, "Id not found (or no read access)".to_string())
  // )
}

#[derive(Serialize, Deserialize)]
struct PasswordForgot {
  email: String
}

#[post("/password_forgot", data="<forgot>")]
fn password_forgot(forgot: JSON<PasswordForgot>) -> Result<JSON<Value>, status::Custom<String>> {
  db_call!(
    "SELECT password_forgot(user_email := $1);",
    &[&forgot.email],
    status::Custom(Status::MethodNotAllowed, "Id not found (or no read access)".to_string())
  )
}

#[derive(Serialize, Deserialize)]
struct PasswordReset {
  token: String,
  password: String
}


#[post("/password_reset", data="<reset>")]
fn password_reset(reset: JSON<PasswordReset>) -> Result<JSON<Value>, status::Custom<String>> {
  db_call!(
    "SELECT password_reset(reset_token := $1, new_password := $2);",
    &[&reset.token, &reset.password],
    status::Custom(Status::MethodNotAllowed, "Id not found (or no read access)".to_string())
  )
}


#[error(400)]
fn badrequest() -> JSON<Value> { 
  JSON(Value::from_str("{\"error\":\"An error occurred on the server\"}").unwrap())
}

#[error(401)]
fn unauthorized() -> JSON<Value> { 
  JSON(Value::from_str("{\"error\":\"No Authorization header found\"}").unwrap())
}

#[error(404)]
fn notfound() -> JSON<Value> { 
  JSON(Value::from_str("{\"error\":\"Resource not found\"}").unwrap())
}



fn main() {
    rocket::ignite().mount("/", routes![
      login,
      people_add, people_get, people_get_i, people_set_i,
      roles_add, roles_get, roles_get_i, roles_set,
      permissions_get,
      fields_get, fields_get_i, fields_set,
      password_forgot, password_reset,

    ])
    .catch(errors![
      badrequest,
      unauthorized,
      notfound
    ])
    .launch();
}