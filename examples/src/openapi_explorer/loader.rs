pub async fn load_source(input: &str) -> Result<(String, String), String> {
    if input.starts_with("http://") || input.starts_with("https://") {
        let response = reqwest::get(input)
            .await
            .map_err(|error| error.to_string())?;
        let response = response
            .error_for_status()
            .map_err(|error| error.to_string())?;
        let text = response.text().await.map_err(|error| error.to_string())?;
        Ok((input.to_string(), text))
    } else {
        let text = std::fs::read_to_string(input).map_err(|error| error.to_string())?;
        Ok((input.to_string(), text))
    }
}
