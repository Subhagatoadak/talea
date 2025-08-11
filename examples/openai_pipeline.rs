use talea::openai_pipeline::OpenAiPipeline;

fn main() {
    // This example expects the OPENAI_API_KEY environment variable to be set.
    let pipeline = OpenAiPipeline::from_env("gpt-3.5-turbo").expect("OPENAI_API_KEY must be set");
    let response = pipeline
        .generate("Write a short greeting from Talea.")
        .expect("failed to generate text");
    println!("{}", response);
}
