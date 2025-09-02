# tzh

An AI-powered translation tool.

This tool is quite simple. It just calls the LLM API to translate text in the command line.

## Example

- configure base url, model and api key

```bash
tzh c \
--endpoint https://api.deepseek.com \
--model deepseek-chat \
--api-key YOUR_API_KEY
```

- translate text using parameters

```bash
tzh t Hello World
```

- translate text from stdin

```bash
cat input.txt | tzh t
```

- translate text line by line (streamed)

```bash
cat input.txt | tzh t -s
```

- translate text with plain style output

```bash
cat input.txt | tzh t -p
```

- help to see usage of other options

```bash
tzh -h
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
