use askama::Template;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Link {
    pub src: String,
    pub dst: String,
}

#[derive(Template, Serialize, Deserialize)]
#[template(path = "main.html")]
pub struct MainPage {
    pub links: Vec<Link>,
}

#[derive(Template)]
#[template(path = "404.html")]
pub struct FourOhFour {}
