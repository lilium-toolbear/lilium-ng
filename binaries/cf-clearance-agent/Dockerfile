FROM mcr.microsoft.com/playwright:v1.61.1-noble

ENV HOME=/home/pwuser
ENV NODE_ENV=production
WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci --omit=dev --ignore-scripts

COPY src ./src

RUN test -x /usr/bin/Xvfb \
    && mkdir -p /data/chrome-profile \
    && chown -R pwuser:pwuser /app /data/chrome-profile

USER pwuser

EXPOSE 8787
VOLUME ["/data/chrome-profile"]

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD node -e "fetch('http://127.0.0.1:8787/healthz').then(response => process.exit(response.ok ? 0 : 1)).catch(() => process.exit(1))"

CMD ["node", "src/main.mjs"]
