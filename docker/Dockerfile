FROM rust:1.79-slim AS builder

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev

WORKDIR /app

# 复制项目文件
COPY . .

RUN cargo build -r --bin nitro-attest

FROM rust:1.79-slim AS runner

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    libssl-dev

WORKDIR /app

# 复制二进制文件
COPY --from=builder /app/target/release/nitro-attest /app/nitro-attest

# 设置可执行权限（如果需要的话）
RUN chmod +x /app/nitro-attest

# 运行命令
CMD ["/app/nitro-attest"]