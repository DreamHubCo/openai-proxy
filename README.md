# openai-proxy

Deploy a proxy in front of OpenAI API to allow your frontend to make requests without exposing your API key while enforcing rate limiting. Designed for speed and ease of deployment.

## Features

- [x] JWT authentication
- [x] Chat streaming
- [ ] Per user rate limiting

## Run the example

1. Start the proxy
    ```bash
    export OPENAI_API_KEY=sk-XXX
    cargo run
    ```
1. Start the example
    ```bash
    cd examples/openai-proxy-js-example
    npm i
    npm run dev
    open localhost:3000
    ```
