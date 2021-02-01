---
title: 'Configuring Rover'
sidebar_title: 'Configuring'
---

## Authenticating with Apollo Studio

### 1. Obtain an API key

All Rover commands that communicate with [Apollo Studio](https://www.apollographql.com/docs/studio/) require an API key to do so. Studio supports two types of API keys: **personal API keys** and **graph API keys**.

* **On your local development machine,** use a personal API key.
* **In shared environments like CI,** use a graph API key.

> [Learn how to obtain an API key](https://www.apollographql.com/docs/studio/api-keys/)

### 2. Provide the API key to Rover

You can provide your API key to Rover either via a [Rover command](#via-the-auth-command) (recommended for local development) or by setting an [environment variable](#with-an-environment-variable) (recommended for automation and CI).

> If you provide an API key via _both_ methods, the environment variable takes precedence.

#### Via the `auth` command

You can provide your API key to Rover by running the following command:

```shell
rover config auth
```

This method is recommended for local development. If you have more than one API key you want to use with Rover, you can assign those keys to different [configuration profiles](#configuration-profiles).

> The `auth` command is **interactive** to prevent your API key from appearing in your terminal command history. Because it's interactive, we recommend using an [environment variable](#with-an-environment-variable) in automated environments such as CI. 

#### With an environment variable

You can provide your API key to Rover by setting it as the value of the `APOLLO_KEY` environment variable. This method is recommended for automated environments such as CI.

## Configuration profiles

You can create multiple **configuration profiles** in Rover. Each configuration profile has its own associated API key, so you can use different configuration profiles when interacting with different graphs.

To specify which configuration profile to use for a particular command, use the `--profile` flag:

```shell
rover graph check my-company@prod --profile work
```

If you don't specify a configuration profile for a command, Rover uses the default profile (named `default`).

To view all commands for working with configuration profiles, run the following command:

```
rover config --help
```

## Logging

Rover supports the following levels of logging, in descending order of severity:

* `error`
* `warn`
* `info`
* `debug`
* `trace`

By default, Rover only logs `error`, `warn`, and `info` messages. You can configure this behavior for a command by setting its minimum log level with the `--log` flag:

```
rover graph check my-graph@prod --schema ./schema.graphql --log debug
```

If Rover log messages are unhelpful or unclear, please leave us feedback in an 
[issue on GitHub](https://github.com/apollographql/rover/issues)!

## Setting config storage location

Rover stores your configuration in a local file and uses it when making requests. By default, this file is stored in your operating system's default configuration directory, in a file named `.sensitive`.

You can override the location of this configuration file by setting the `APOLLO_CONFIG_HOME` environment variable. This can be useful for CI systems that don't give you access to default operating system directories.

```
# Stores config in ./myspecialconfig/rover.toml
APOLLO_CONFIG_HOME=./myspecialconfig/
```

## All supported environment variables

You can configure Rover's behavior by setting the environment variables listed below.

If present, an environment variable's value takes precedence over all other methods of configuring the associated behavior.

| Name                        | Value          |
|-----------------------------|----------------|
| `APOLLO_CONFIG_HOME` | The path where Rover's configuration is stored. The default value is your operating system's default configuration directory. |
| `APOLLO_KEY` | The API key that Rover should use to authenticate with Apollo Studio. |
| `APOLLO_TELEMETRY_DISABLED` | Set to `1` if you don't want Rover to collect anonymous usage data. |
| `APOLLO_VCS_BRANCH` | The name of the branch in your version control system (VCS) |
| `APOLLO_VCS_COMMIT` | The short hash or unique ID of the commit in your VCS |
| `APOLLO_VCS_COMMITTER` | The author identifier for a commit (i.e. "Jane Doe <jane@example.com>") |
| `APOLLO_VCS_REMOTE_URL` | The remote url for your VCS repository |
