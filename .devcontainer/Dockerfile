FROM alpine:3.19

RUN apk update
RUN apk add cargo rust mold watchexec
# needed by sqlx
RUN apk add pkgconfig openssl-dev 
# TODO: we will use postgres
RUN apk add sqlite

ENV PATH=$PATH:/root/.cargo/bin