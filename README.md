# TTS-service

HTTP microservice using Axum to generate TTS from an HTTP reqwest.

## Modes
- eSpeak - Local TTS, low quality. Returns WAV audio.
- gTTS - Cloud TTS, medium quality. Returns MP3 audio
- gcloud - Google Cloud TTS, high quality. Returns OPUS audio. **Requires a gCloud API key**
- Polly - Amazon Polly TTS, high quality. Returns OggVorbis audio. **Requires Amazon Polly credentials**

## Supported endpoints:
- `GET /tts?text={CONTENT}&lang={VOICE}&mode={MODE}&speaking_rate={SPEAKING_RATE}&max_length={MAX_LENGTH}&preferred_format={PREFERRED_AUDIO_FORMAT}` - Returns the audio generated.
- `GET /voices?mode={MODE}&raw={BOOL}` - Returns the supported voices for the given mode as either a JSON array of strings, or a raw format from the source with the `raw` set to true.
- `GET /modes` - Returns the currently supported modes for TTS as a JSON array of strings.

## Error Codes:
Non-200 responses will return a JSON object with the following keys:

### `code` - int
- `0` - Unknown error
- `1` - Unknown voice
- `2` - Max length exceeded
- `3` - Speaking rate exceeded limits, see the `display` for more information
- `4` - `AUTH_KEY` has been set and the `Authorization` header doesn't match the key.
### `display` - str
A human readable message describing the error

## Docker Setup

### Build
```bash
docker build -t tts-service:local .
```

### Run with Docker Compose
```bash
docker-compose up
```

This will:
- Build the image with tag `tts-service:local`
- Start the service on port 3000
- Load environment variables from `.env`

### Run with Docker directly
```bash
docker run -p 3000:3000 --env-file .env tts-service:local
```

## Environment Variables (default)
- `BIND_ADDR`(`0.0.0.0:3000`) - The address to bind the web server to

- `LOG_LEVEL`(`INFO`) - The lowest log level to output to stdout

- `AUTH_KEY` - If set, this key must be sent in the `Authorization` header of each request

### DeepL API (for translation features)
- `DEEPL_API_TIER` - DeepL API tier (`Free` for free tier or `Pro` for pro tier). Defaults to `Free`

- `DEEPL_KEY` - DeepL API key

### gTTS Required
- `IPV6_BLOCK` - A block of IPv6 addresses, randomly selected for each gTTS request

### gCloud Required
- `GOOGLE_APPLICATION_CREDENTIALS` - The file path to the gCloud JSON

### Polly Required
- `AWS_REGION` - The AWS region to use

- `AWS_ACCESS_KEY_ID` - The AWS access key ID

- `AWS_SECRET_ACCESS_KEY` - The AWS secret access key
