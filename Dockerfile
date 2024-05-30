FROM rust:1.78-bookworm AS rust-builder

WORKDIR /app

COPY src/ src
COPY Cargo.* .
RUN cargo build --release

# ----------------------------------------------------------------------------

FROM cgr.dev/chainguard/python:latest-dev as py-builder

WORKDIR /app

RUN pip install boto3 --no-cache-dir --user

# ----------------------------------------------------------------------------

FROM cgr.dev/chainguard/python:latest

WORKDIR /app

COPY aws-config /opt/aws-config
COPY --from=rust-builder --chown=nonroot:nonroot /app/target/release/epicac /opt/epicac

# Make sure you update Python version in path
COPY --from=py-builder /home/nonroot/.local/lib/python3.12/site-packages /home/nonroot/.local/lib/python3.12/site-packages
COPY example.py .
