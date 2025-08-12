use talea::openai_pipeline::OpenAiPipeline;

fn main() {
    // This example expects the OPENAI_API_KEY environment variable to be set.
    OpenAiPipeline::quick(
        "Explain the significance of the Renaissance.",
        "renaissance.txt",
    )
    .expect("failed to generate text");
}
