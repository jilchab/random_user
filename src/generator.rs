use crate::types::{Gender, Nationality, RandomUser, RandomUserResponse, RandomUserResult};
use thiserror::Error;

/// Helper to request users with filters like gender, nationalities, etc.
pub struct UserGeneratorBuilder {
    req: reqwest::RequestBuilder,
}

impl UserGeneratorBuilder {
    pub(crate) fn new(req: reqwest::RequestBuilder) -> Self {
        Self { req }
    }
    /// Request a specific gender
    pub fn gender(self, gender: Gender) -> Self {
        Self::new(
            self.req
                .query(&[("gender", serde_json::to_value(gender).unwrap().as_str())]),
        )
    }

    /// Request a specific nationality
    pub fn nationality(self, nationality: Nationality) -> Self {
        Self::new(
            self.req
                .query(&[("nat", serde_json::to_value(nationality).unwrap().as_str())]),
        )
    }

    /// Request specific nationalities, picked at random between each user
    pub fn nationalities(self, nationalities: &[Nationality]) -> Self {
        let mut nats = String::new();
        for nat in nationalities {
            nats += serde_json::to_value(nat).unwrap().as_str().unwrap();
            nats.push(',');
        }
        nats.pop();
        Self::new(self.req.query(&[("nat", nats)]))
    }

    /// Request with a specified seed, allow to always generate the same users
    ///
    /// ### Warning:
    /// May discard other filters
    pub fn seed(self, seed: &str) -> Self {
        Self::new(self.req.query(&[("seed", seed)]))
    }

    /// Request a user with specific password rules
    ///
    /// Format, without spaces:
    /// `CHARSETS,MIN_LENGTH-MAX_LENGTH`
    /// or
    /// `CHARSETS,MAX_LENGTH`
    ///
    /// You can mix and match the charsets below for the CHARSETS option above:
    /// ```md
    /// special    !"#$%&'()*+,- ./:;<=>?@[\]^_`{|}~
    /// upper      ABCDEFGHIJKLMNOPQRSTUVWXYZ
    /// lower      abcdefghijklmnopqrstuvwxyz
    /// number     0123456789
    /// ```
    /// `MIN_LENGTH` and `MAX_LENGTH` are the min/max length of the passwords that you want to generate.
    /// By default, passwords will be between 8 - 64 characters long.
    ///
    /// ## Example:
    /// ```
    /// // Get a user with a password composed with 8 upper and/or lower characters
    /// let user = generator.get().password("upper,lower,8").fetch_one().await?
    /// ```
    pub fn password(self, charset: &str) -> Self {
        Self::new(self.req.query(&[("password", charset)]))
    }

    /// Generate users with the api informations
    pub async fn fetch_with_info(self, count: usize) -> Result<RandomUserResult> {
        self.count(count).request().await
    }

    /// Generate users
    pub async fn fetch(self, count: usize) -> Result<Vec<RandomUser>> {
        Ok(self.count(count).request().await?.results)
    }

    /// Generate 1 user
    pub async fn fetch_one(self) -> Result<RandomUser> {
        Ok(self.fetch(1).await?.remove(0))
    }

    fn count(self, count: usize) -> Self {
        Self::new(self.req.query(&[("results", count)]))
    }

    async fn request(self) -> Result<RandomUserResult> {
        let api_rsp = self.req.send().await?;
        let rsp = Self::parse_response(api_rsp).await?;
        match rsp {
            RandomUserResponse::Error(e) => Err(RandomUserError::Api(e)),
            RandomUserResponse::Result(res) => Ok(res),
        }
    }

    async fn parse_response(response: reqwest::Response) -> Result<RandomUserResponse> {
        let content_type = response
            .headers()
            .get("content-type")
            .ok_or(RandomUserError::BadFormat)?
            .to_str()
            .map_err(|_| RandomUserError::BadFormat)?
            .to_owned();
        let text = response.text().await?;
        match content_type {
            ct if ct.contains("text/plain") => Ok(RandomUserResponse::Error(text)),
            ct if ct.contains("application/json") => {
                serde_json::from_str::<RandomUserResponse>(&text)
                    .map_err(|_| RandomUserError::BadFormat)
            }
            _ => Err(RandomUserError::BadFormat),
        }
    }
}

/// Random user generator
///
/// ## Example:
/// ```
/// let generator = UserGenerator::new();
///
/// let user = generator.fetch_one().await?
///
/// println!("{:#?}", user);
/// ```
pub struct UserGenerator {
    client: reqwest::Client,
}

impl UserGenerator {
    const API_URL: &str = "https://randomuser.me/api/1.4/";

    #[must_use]
    pub fn new() -> UserGenerator {
        UserGenerator {
            client: reqwest::Client::new(),
        }
    }

    /// Start the request to easily apply filters
    #[must_use]
    pub fn get(&self) -> UserGeneratorBuilder {
        UserGeneratorBuilder::new(self.client.get(Self::API_URL))
    }

    /// Generate users with the api informations
    ///
    /// ## Example:
    /// ```
    /// // Fetch 5 random users with api info
    /// let users = generator.fetch_with_info(5).await?
    ///
    /// println!("{:?}", users.info);
    /// for user in users.results {
    ///     prinln!("{user:?}");
    /// }
    ///
    /// ```
    pub async fn fetch_with_info(&self, count: usize) -> Result<RandomUserResult> {
        self.get().fetch_with_info(count).await
    }

    /// Generate users
    ///
    /// ## Example:
    /// ```
    /// // Fetch 5 random users
    /// let users = generator.fetch(5).await?
    ///
    /// for user in users {
    ///     prinln!("{user:?}");
    /// }
    ///
    /// ```
    pub async fn fetch(&self, count: usize) -> Result<Vec<RandomUser>> {
        self.get().fetch(count).await
    }

    /// Generate a user
    ///
    /// ## Example:
    /// ```
    /// let user = generator.fetch_one().await?
    /// println("{user:?}");
    /// ```
    pub async fn fetch_one(&self) -> Result<RandomUser> {
        self.get().fetch_one().await
    }
}

impl Default for UserGenerator {
    fn default() -> Self {
        Self::new()
    }
}

type Result<T> = std::result::Result<T, RandomUserError>;

#[derive(Debug, Error)]
pub enum RandomUserError {
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Api error: {0}")]
    Api(String),
    #[error("Bad format")]
    BadFormat,
}
