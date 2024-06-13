use crate::models::conversation::Conversation;
use cfg_if::cfg_if;
use leptos::*;

static CHARACTER_NAME: &str = "### Assistant";
static USER_NAME: &str = "### Human";

#[server(Converse, "/api")]
pub async fn converse(prompt: Conversation) -> Result<String, ServerFnError> {
    use actix_web::dev::ConnectionInfo;
    use actix_web::web::Data;
    use leptos_actix::extract;
    use llm::models::Llama;
    let model: Llama = get_language_model();
    // let model =
    //     extract(|data: Data<Llama>, _connection: ConnectionInfo| async { data.into_inner() })
    //         .await
    //         .unwrap();

    use llm::KnownModel;
    // use tokio::runtime::Builder;

    // let runtime = Builder::new_multi_thread()
    //     .worker_threads(4)
    //     .thread_name("my-custom-name")
    //     .thread_stack_size(3 * 1024 * 1024)
    //     .build()
    //     .unwrap();
    let character_name = "### Assistant";
    let user_name = "### Human";
    let persona = "A chat between a human and an assistant";
    let mut history = format!(
        "{character_name}: Hello may I help you today? \n\
      {user_name}: What is the capital of France \n\
      {character_name}: Paris is the capital of France. \n
      "
    );

    for message in prompt.messages.into_iter() {
        let msg = message.text;
        let curr_line = if message.user {
            format!("{user_name}: {msg}\n")
        } else {
            format!("{character_name}: {msg}\n")
        };

        history.push_str(&curr_line);
    }

    let mut res = String::new();
    let mut rng = rand::thread_rng();
    let mut buf = String::new();

    let mut session = model.start_session(Default::default());
    dbg!(format!(
        "\
{persona}
{history}
{character_name}:"
    ));
    session
        .infer(
            &model,
            &mut rng,
            &llm::InferenceRequest {
                prompt: format!(
                    "\
{persona}
{history}
{character_name}:"
                )
                .as_str()
                .into(),
                parameters: &llm::InferenceParameters::default(),
                play_back_previous_tokens: false,
                maximum_token_count: None,
            },
            &mut Default::default(),
            inference_callback(String::from(user_name), &mut buf, &mut res),
        )
        .unwrap_or_else(|e| panic!("{e}"));

    // match result {
    //     Ok(fine) => println!("\n\nInference stats:\n{fine}"),
    //     Err(err) => println!("\n{err}"),
    // }

    Ok(res)
}
cfg_if! {
    if #[cfg(feature = "ssr")] {
        use llm::models::Llama;
        use std::convert::Infallible;
        fn inference_callback<'a>(
            stop_sequence: String,
            buf: &'a mut String,
            out_str: &'a mut String,
        ) -> impl FnMut(llm::InferenceResponse) -> Result<llm::InferenceFeedback, Infallible> + 'a {
            use llm::InferenceFeedback::Halt;
            use llm::InferenceFeedback::Continue;

            move |resp| match resp {
                llm::InferenceResponse::InferredToken(t) => {
                    print!("inside here");
                    let mut reverse_buf = buf.clone();
                    reverse_buf.push_str(t.as_str());
                    if stop_sequence.as_str().eq(reverse_buf.as_str()) {
                        buf.clear();
                        return Ok::<llm::InferenceFeedback, Infallible>(Halt);
                    } else if stop_sequence.as_str().starts_with(reverse_buf.as_str()) {
                        buf.push_str(t.as_str());
                        return Ok(Continue);
                    }

                    if buf.is_empty() {
                        out_str.push_str(&t);
                    } else {
                        out_str.push_str(&reverse_buf);
                    }

                    Ok(Continue)
                }
                llm::InferenceResponse::EotToken => {
                    print!("inside here 2");
                    Ok(Halt)
                },
                _ => {
                    print!("inside here 3");
                    Ok(Continue)
                },
            }
        }

        fn get_language_model() -> Llama {
            use actix_web::*;
            use std::env;
            use dotenv::dotenv;
                use std::path::PathBuf;

                dotenv().ok();
                let model_path = env::var("MODEL_PATH").expect("Model path must be set");

                llm::load::<Llama>(
                    &PathBuf::from(model_path.clone()),
                    llm::TokenizerSource::Embedded,
                    Default::default(),
                    llm::load_progress_callback_stdout,
                ).unwrap_or_else(move |err| {
                    panic!("Failed to load model from {model_path:?}: {}", err)
                })
            }
    }

}
