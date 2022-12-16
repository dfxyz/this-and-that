FROM alpine

RUN ["apk", "update"]
RUN ["apk", "add", "transmission-daemon"]

ENTRYPOINT ["transmission-daemon", "-f"]
