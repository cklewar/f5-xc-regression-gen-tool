# Project: {{ config.project.module }}

# Actions

## EUTs:

{% for site in actions.sites -%}
### {{ site }}

```bash
curl --request POST \
     --form token="TOKEN" \
     --form ref=main \
     --form "variables[URL]=URL" \
     --form "variables[ACTION]=deploy-{{ site }}" \
     "https://gitlab.com/api/v4/projects/46168301/trigger/pipeline"
```

```bash
curl --request POST \
     --form token="TOKEN" \
     --form ref=main \
     --form "variables[URL]=URL" \
     --form "variables[ACTION]=destroy-{{ site }}" \
     "https://gitlab.com/api/v4/projects/46168301/trigger/pipeline"
```
{% endfor %}

## FEATURES:

{% for feature in actions.features -%}
### {{ feature }}

```bash
curl --request POST \
     --form token="TOKEN" \
     --form ref=main \
     --form "variables[URL]=URL" \
     --form "variables[ACTION]=deploy-{{ feature }}" \
     "https://gitlab.com/api/v4/projects/46168301/trigger/pipeline"
```

```bash
curl --request POST \
     --form token="TOKEN" \
     --form ref=main \
     --form "variables[URL]=URL" \
     --form "variables[ACTION]=destroy-{{ feature }}" \
     "https://gitlab.com/api/v4/projects/46168301/trigger/pipeline"
```

{% endfor %}

## RTEs:

{% for rte in actions.rtes -%}
### {{ rte }}

```bash
curl --request POST \
     --form token="TOKEN" \
     --form ref=main \
     --form "variables[URL]=URL" \
     --form "variables[ACTION]=deploy-{{ rte }}" \
     "https://gitlab.com/api/v4/projects/46168301/trigger/pipeline"
```

```bash
curl --request POST \
     --form token="TOKEN" \
     --form ref=main \
     --form "variables[URL]=URL" \
     --form "variables[ACTION]=destroy-{{ rte }}" \
     "https://gitlab.com/api/v4/projects/46168301/trigger/pipeline"
```

{% endfor %}

## TESTS:

{% for test in actions.tests -%}
### {{ test }}

```bash
curl --request POST \
     --form token="TOKEN" \
     --form ref=main \
     --form "variables[URL]=URL" \
     --form "variables[ACTION]=deploy-{{ test }}" \
     "https://gitlab.com/api/v4/projects/46168301/trigger/pipeline"
```

```bash
curl --request POST \
     --form token="TOKEN" \
     --form ref=main \
     --form "variables[URL]=URL" \
     --form "variables[ACTION]=destroy-{{ test }}" \
     "https://gitlab.com/api/v4/projects/46168301/trigger/pipeline"
```

{% endfor %}

## VERIFICATIONS:

{% for verification in actions.verifications -%}
### {{ verification }}

```bash
curl --request POST \
     --form token="TOKEN" \
     --form ref=main \
     --form "variables[URL]=URL" \
     --form "variables[ACTION]=deploy-{{ verification }}" \
     "https://gitlab.com/api/v4/projects/46168301/trigger/pipeline"
```

```bash
curl --request POST \
     --form token="TOKEN" \
     --form ref=main \
     --form "variables[URL]=URL" \
     --form "variables[ACTION]=destroy-{{ verification }}" \
     "https://gitlab.com/api/v4/projects/46168301/trigger/pipeline"
```

{% endfor %}
