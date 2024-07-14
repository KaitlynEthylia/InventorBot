use anyhow::Result;

// https://random-word-form.herokuapp.com/random/noun
pub fn gen_item() -> Result<String> {
	log::info!("Generating random invention");
	let response = &super::RQ_CLIENT
		.get()
		.unwrap()
		.get("https://random-word-form.herokuapp.com/random/noun")
		.send()?
		.text()?;
	let [word] = serde_json::from_str::<[&str; 1]>(response)?;

	let mut str = String::from(match rand::random::<bool>() {
		true => "the ",
		false => "",
	});

	str.push_str(word);

	Ok(str)
}
