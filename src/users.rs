use crate::cosmosdb::UserDb;
use crate::models::{PartialUser, User};
use crate::utility::{COLLECTION_NAME, DATABASE_NAME};
/**
 * this module implements the WebApi to create the database/collection, list all the users, and to create/find/delete
 * a User document in CosmosDb
 */
use actix_web::{web, HttpResponse};
use log::trace;

/**
 *  this will get a list of all documents.  Note this does *not* do pagination. This would be a reasonable next step to
 *  show in the sample
 */
pub async fn list_users() -> HttpResponse {
    //
    //  this match should always succeed as it is tested in main()
    let userdb = UserDb::new().await;

    // Get list of users
    match userdb.list().await {
        Ok(users) => HttpResponse::Ok()
            .content_type("application/json")
            .json(users),
        Err(err) => HttpResponse::NotFound()
            .content_type("text/plain")
            .body(format!("Failed to retrieve user list: {}", err)),
    }
}
/**
 *  this will get a list of all documents.  Note this does *not* do pagination. This would be a reasonable next step to
 *  show in the sample
 */
pub async fn find_user_by_id(id: web::Path<String>) -> HttpResponse {
    //
    //  this match should always succeed as it is tested in main()
    let userdb = UserDb::new().await;

    // Get list of users
    match userdb.find_user(&id).await {
        Ok(user) => HttpResponse::Ok()
            .content_type("application/json")
            .json(user),
        Err(err) => HttpResponse::NotFound()
            .content_type("text/plain")
            .body(format!("Failed to retrieve user list: {}", err)),
    }
}
/**
 * this sets up CosmosDb to make the sample run. the only prereq is the secrets set in
 * .devconainter/required-secrets.json, this API will call setupdb. this just calls the setupdb api and deals with errors
 */
pub async fn setup() -> HttpResponse {
    let userdb = UserDb::new().await;
    match userdb.setupdb().await {
        Ok(..) => HttpResponse::Ok().content_type("text/plain").body(format!(
            "database: {} collection: {} \ncreated",
            DATABASE_NAME, COLLECTION_NAME
        )),
        Err(err) => HttpResponse::BadRequest()
            .content_type("text/plain")
            .body(format!("Failed to create database: {}", err.to_string())),
    }
}

/**
 *  this creates a user.  it uses web forms to collect the data from the client.  Note that if you are using PostMan
 *  to call this API, set the form data in 'x-www-form-urlencoded', *not* in 'form-data', as that will fail with a
 *  hard-to-figure-out error in actix_web deserialize layer.
 */
pub async fn create(user_req: web::Form<PartialUser>) -> HttpResponse {
    let pp: PartialUser = user_req.into_inner();
    let user: User = pp.into();
    let userdb = UserDb::new().await;
    match userdb.create_user(user.clone()).await {
        Ok(..) => {
            let user_json = match serde_json::to_string(&user) {
                Ok(json) => json,
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .body(format!("Failed to serialize user: {}", e.to_string()))
                }
            };
            HttpResponse::Ok()
                .content_type("application/json")
                .body(user_json)
        }
        Err(e) => HttpResponse::BadRequest()
            .content_type("text/plain")
            .body(format!("Failed to add user to database: {}", e.to_string())),
    }
}

pub async fn delete(id: web::Path<String>) -> HttpResponse {
    // match parse_number(&id.into_inner()) {
    //     Ok(number) => HttpResponse::Ok().body(format!("Parsed number: {}", number)),
    //     Err(_) => HttpResponse::BadRequest().body("Invalid number format"),
    // }

    // let id_str = id.into_inner();
    // let id: u64 = id_str.parse().expect("Failed to parse string as u64");

    // // Find the index of the user with the given ID
    // let index = users.iter().position(|user| user.id == id_str);

    // // If a user with the given ID exists, remove them
    // if let Some(index) = index {
    //     users.remove(index);
    // } else {
    //     return HttpResponse::NotFound().finish();
    // }
    trace!("id :{:?}", id);
    HttpResponse::BadRequest().body("Delete not implemented yet")
}

// const PLAYER_TEST_DATA: &'static str = r#"
// [
//     {
//         "id": "user1",
//         "email": "test@test.com",
//         "name": "Alice",
//         "foreground": "Red",
//         "background": "Blue",
//         "picture": "https://example.com/image1.jpg"
//     },
//     {
//         "id": "user2",
//         "email": "test@test.com",
//         "name": "Bob",
//         "foreground": "Green",
//         "background": "Yellow",
//         "picture": "https://example.com/image2.jpg"
//     },
//     {
//         "id": "user3",
//         "email": "test@test.com",
//         "name": "Charlie",
//         "foreground": "Blue",
//         "background": "White",
//         "picture": "https://example.com/image3.jpg"
//     },
//     {
//         "id": "user4",
//         "email": "test@test.com",
//         "name": "Dave",
//         "foreground": "Yellow",
//         "background": "Red",
//         "picture": "https://example.com/image4.jpg"
//     },
//     {
//         "id": "user5",
//         "email": "test@test.com",
//         "name": "Eve",
//         "foreground": "White",
//         "background": "Green",
//         "picture": "https://example.com/image5.jpg"
//     }
// ]
// "#;
