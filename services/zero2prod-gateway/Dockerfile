FROM nginx:stable
RUN rm -f /etc/nginx/conf.d/default.conf
RUN ln -sf /dev/stdout /var/log/nginx/access.log && ln -sf /dev/stderr /var/log/nginx/error.log
COPY ./zero2prod.conf /etc/nginx/conf.d/default.conf
COPY ./snippets/ssl-params.conf /etc/nginx/snippets/ssl-params.conf
COPY ./snippets/ssl-dhparams.pem /etc/nginx/snippets/ssl-dhparams.pem
COPY ./certs /etc/nginx/certs
