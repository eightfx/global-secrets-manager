use global_secrets_manager::GlobalSecretsManager;

#[derive(GlobalSecretsManager)]
#[derive(Debug, serde::Deserialize)]
pub struct my_secrets{
	twitter_api_key: String,
	twitter_api_secret: String,
}



fn main(){
	dbg!(&my_secrets.twitter_api_key);
	dbg!(&my_secrets.twitter_api_secret);
}
