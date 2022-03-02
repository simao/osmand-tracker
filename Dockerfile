FROM alpine:latest

# RUN apk update && apk add

WORKDIR /opt/osmand-tracker

ENV PATH=${PATH}:/opt/osmand-tracker
ENV DEV_LOG=0

COPY target/x86_64-unknown-linux-musl/release/osmand-tracker /opt/osmand-tracker

COPY dist /opt/osmand-tracker/dist

EXPOSE 9001

CMD ["/opt/osmand-tracker/osmand-tracker"]
