# Build stage
FROM rust:latest AS builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    cmake \
    build-essential \
    pkg-config \
    git \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.toml

# Copy source tree
COPY src src

# Build for release
RUN cargo build --release

# Build MBROLA from source
FROM builder AS mbrola-builder

WORKDIR /mbrola-build

RUN git clone https://github.com/numediart/MBROLA.git . && \
    make && \
    cp Bin/mbrola /usr/local/bin/mbrola

# Download MBROLA voice databases
RUN git clone https://github.com/numediart/MBROLA-voices.git /mbrola-voices && \
    mkdir -p /usr/local/share/espeak-ng-data/voices/mb && \
    for voice in /mbrola-voices/*/; do \
        voice_name=$(basename "$voice"); \
        ln -s "$voice" "/usr/local/share/espeak-ng-data/voices/mb/$voice_name"; \
    done

# Runtime stage
FROM debian:unstable-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    espeak-ng-data \
    espeak-ng \
    && rm -rf /var/lib/apt/lists/*

# Copy MBROLA binary and voices from builder
COPY --from=mbrola-builder /usr/local/bin/mbrola /usr/local/bin/mbrola
COPY --from=mbrola-builder /mbrola-voices /usr/local/share/mbrola/voices
COPY --from=mbrola-builder /usr/local/share/espeak-ng-data /usr/local/share/espeak-ng-data

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/tts-service /app/tts-service

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/modes || exit 1

# Run the application
CMD ["./tts-service"]
