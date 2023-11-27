use actix_web::{web, App, HttpResponse, HttpServer};
use bson::{doc, Bson};
use futures_util::stream::TryStreamExt;
use mongodb::error::Error;

use mongodb::{bson::oid::ObjectId, options::ClientOptions, Client};
use serde::{Deserialize, Serialize};
use std::convert::From;

#[derive(Debug, Serialize, Deserialize)]
struct Item {
    #[serde(rename = "_id")]
    id: Option<ObjectId>,
    name: String,
    description: String,
}
async fn add_item(item: web::Json<Item>) -> HttpResponse {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .expect("Failed to parse MongoDB connection string");
    let client = Client::with_options(client_options).expect("Failed to create MongoDB client");

    let db = client.database("mydb");
    let collection = db.collection::<Item>("items");

    let mut inserted_item = item.into_inner(); // Convertir el objeto JSON en un Item
    inserted_item.id = Some(ObjectId::new()); // Asignar un nuevo ObjectId al campo id

    match collection.insert_one(inserted_item, None).await {
        Ok(_) => HttpResponse::Ok().body("Item added successfully"),
        Err(e) => {
            // Devuelve el error MongoDB como parte de la respuesta JSON
            HttpResponse::InternalServerError().json(format!("Failed to add item: {}", e))
        }
    }
}

/*
async fn add_item(item: web::Json<Item>) -> HttpResponse {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .expect("Failed to parse MongoDB connection string");
    let client = Client::with_options(client_options).expect("Failed to create MongoDB client");

    let db = client.database("mydb");
    let collection = db.collection::<Item>("items");

    let inserted_item = item.into_inner(); // Convertir el objeto JSON en un Item

    match collection.insert_one(inserted_item, None).await {
        Ok(_) => HttpResponse::Ok().body("Item added successfully"),
        Err(e) => {
            // Devuelve el error MongoDB como parte de la respuesta JSON
            HttpResponse::InternalServerError().json(format!("Failed to add item: {}", e))
        }
    }
}
 */
async fn get_all_items() -> HttpResponse {
    let client_options = mongodb::options::ClientOptions::parse("mongodb://localhost:27017")
        .await
        .expect("Failed to parse MongoDB connection string");

    // Resto del código para manejar las operaciones con MongoDB

    // Ejemplo de cómo obtener los elementos usando try_next()
    let items: Result<Vec<Item>, Error> = async {
        let client = mongodb::Client::with_options(client_options)?;
        let db = client.database("mydb");
        let collection = db.collection::<Item>("items");

        let mut cursor = collection.find(None, None).await?;
        let mut items = Vec::new();

        while let Some(result) = cursor.try_next().await? {
            items.push(result);
        }

        Ok(items)
    }
    .await;

    match items {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn delete_item(path: web::Path<String>) -> HttpResponse {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .expect("Failed to parse MongoDB connection string");
    let client = Client::with_options(client_options).expect("Failed to create MongoDB client");

    let db = client.database("mydb");
    let collection = db.collection::<Item>("items");

    let item_id: ObjectId = match path.parse() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid ObjectId"),
    };

    match collection.delete_one(doc! { "_id": item_id }, None).await {
        Ok(result) if result.deleted_count > 0 => {
            HttpResponse::Ok().json("Item deleted successfully")
        }
        _ => HttpResponse::NotFound().json("Item not found"),
    }
}

impl From<Item> for Bson {
    fn from(item: Item) -> Self {
        // Creando un documento BSON con los campos del item
        let doc = doc! {
            "name": item.name,
            "description": item.description,
        };

        // Convirtiendo el documento en un valor Bson
        Bson::Document(doc)
    }
}

async fn edit_item(item: web::Json<Item>) -> HttpResponse {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .expect("Failed to parse MongoDB connection string");
    let client = Client::with_options(client_options).expect("Failed to create MongoDB client");

    let db = client.database("mydb");
    let collection = db.collection::<Bson>("items"); // Cambiar el tipo de la colección a Bson

    let edited_item = item.into_inner(); // Convertir el objeto JSON en un Item
    let id = edited_item.id.clone(); // Obtener el id del item a editar
    let filter = doc! {"_id": id}; // Crear el filtro por id
    let update = doc! {"$set": Bson::from(edited_item)}; // Crear el documento de actualización con el item editado convertido a Bson

    match collection.update_one(filter, update, None).await {
        Ok(_) => HttpResponse::Ok().body("Item edited successfully"),
        Err(e) => {
            // Devuelve el error MongoDB como parte de la respuesta JSON
            HttpResponse::InternalServerError().json(format!("Failed to edit item: {}", e))
        }
    }
}
/* async fn edit_item(item: web::Json<Item>) -> HttpResponse {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .expect("Failed to parse MongoDB connection string");
    let client = Client::with_options(client_options).expect("Failed to create MongoDB client");

    let db = client.database("mydb");
    let collection = db.collection::<Item>("items");

    let edited_item = item.into_inner(); // Convertir el objeto JSON en un Item
    let id = edited_item.id.clone(); // Obtener el id del item a editar
    let filter = doc! {"_id": id}; // Crear el filtro por id
    let update = doc! {"$set": edited_item}; // Crear el documento de actualización con el item editado

    match collection.update_one(filter, update, None).await {
        Ok(_) => HttpResponse::Ok().body("Item edited successfully"),
        Err(e) => {
            // Devuelve el error MongoDB como parte de la respuesta JSON
            HttpResponse::InternalServerError().json(format!("Failed to edit item: {}", e))
        }
    }
} */

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(web::resource("/edit").route(web::post().to(edit_item))) // Ruta para editar items
            .route("/add", web::post().to(add_item))
            .route("/", web::get().to(get_all_items))
            .route("/delete/{id}", web::delete().to(delete_item))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
