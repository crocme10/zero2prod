# Error Handling

## Anyhow, Snafu, Thiserror

There are many choices for an error library. In the past
I've used Snafu with success. A majority of developer are using anyhow.
More recently I started to use error-stack... They all have pros and cons,
but rolling your own error handling story is not such a big deal, the
result is customized for your needs, and you drop a dependency. 
