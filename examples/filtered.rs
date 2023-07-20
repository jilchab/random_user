use random_user::{Gender, Nationality, UserGenerator};

#[tokio::main]
async fn main() {
    let generator = UserGenerator::new();

    // Get 5 Austrialian women with complex passwords
    let users = generator
        .get()
        .gender(Gender::Female)
        .nationality(Nationality::Australian)
        .password("upper,lower,special,12-24")
        .fetch(5)
        .await
        .expect("Error fetching random user");

    for user in users {
        println!(
            "Name: {}, Password: {}",
            user.name.first, user.login.password
        );
    }
}
