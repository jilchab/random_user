use random_user::UserGenerator;

#[tokio::main]
async fn main() {
    let generator = UserGenerator::new();

    let user = generator
        .fetch_one()
        .await
        .expect("Error fetching random user");

    println!("{user:#?}");
}
