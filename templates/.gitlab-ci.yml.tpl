#############################################
# DO NOT EDIT THIS FILE IT IS AUTOGENERATED #
#############################################

stages:
  {% for stage in stages -%}
  - {{ stage }}
  {% endfor %}
variables:
  {% for variable in config.ci.variables -%}
  {{ variable.name | upper }}: "{{ variable.value }}"
  {% endfor -%}
  {% for feature in features -%}
  FEATURE_{{ feature.name | upper }}_ROOT_TF_VAR_FILE: "$FEATURE_ROOT_DIR/{{ feature.name }}/terraform.tfvars"
  {% endfor %}
.deploy_rules:
  rules:
    - if: $ACTION == "deploy" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "deploy" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.destroy_rules:
  rules:
    - if: $ACTION == "destroy" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "destroy" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.deploy_eut_rules:
  rules:
    - if: $ACTION == "deploy-eut" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "deploy-eut" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.destroy_eut_rules:
  rules:
    - if: $ACTION == "destroy-eut" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "destroy-eut" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.deploy_feature_rules:
  rules:
    - if: $ACTION == "deploy-feature" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "deploy-feature" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.destroy_feature_rules:
  rules:
    - if: $ACTION == "destroy-feature" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "destroy-feature" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.deploy_rte_rules:
  rules:
    - if: $ACTION == "deploy-rte" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "deploy-rte" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.destroy_rte_rules:
  rules:
    - if: $ACTION == "destroy-rte" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "destroy-rte" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.deploy_rte_share_rules:
  rules:
    - if: $ACTION == "deploy-rte-share" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "deploy-rte-share" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.destroy_rte_share_rules:
  rules:
    - if: $ACTION == "destroy-rte-share" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "destroy-rte-share" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.regression_test_rules:
  rules:
    - if: $ACTION == "test" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "test" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.regression_verification_rules:
  rules:
    - if: $ACTION == "verify" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "verify" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"
{% for feature in features %}
.deploy_{{ feature.job | replace(from="-", to="_") }}_rules:
  rules:
    - if: $ACTION == "deploy-{{ feature.job }}" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "deploy-{{ feature.job }}" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.destroy_{{ feature.job | replace(from="-", to="_") }}_rules:
  rules:
    - if: $ACTION == "destroy-{{ feature.job }}" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "destroy-{{ feature.job }}" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"
{% endfor -%}
{% for site in eut.sites %}
.deploy_{{ site.job | replace(from="-", to="_") }}_rules:
  rules:
    - if: $ACTION == "deploy-{{ site.job }}" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "deploy-{{ site.job }}" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.destroy_{{ site.job | replace(from="-", to="_") }}_rules:
  rules:
    - if: $ACTION == "destroy-{{ site.job }}" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "destroy-{{ site.job }}" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"
{% endfor -%}
{% for rte in rtes -%}
{% for share in rte.shares %}
.deploy_{{ share.job | replace(from="-", to="_") }}_rules:
  rules:
    - if: $ACTION == "deploy-{{ share.job }}" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "deploy-{{ share.job }}" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.destroy_{{ share.job | replace(from="-", to="_") }}_rules:
  rules:
    - if: $ACTION == "destroy-{{ share.job }}" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "destroy-{{ share.job }}" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"
{% endfor -%}
{% for component in rte.components %}
.deploy_{{ component.job | replace(from="-", to="_") }}_rules:
  rules:
    - if: $ACTION == "deploy-{{ component.job }}" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "deploy-{{ component.job }}" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.destroy_{{ component.job | replace(from="-", to="_") }}_rules:
  rules:
    - if: $ACTION == "destroy-{{ component.job }}" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "destroy-{{ component.job }}" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"
{% endfor -%}
{% for test in rte.tests %}
.regression_{{ test.job | replace(from="-", to="_") }}_rules:
  rules:
    - if: $ACTION == "test-{{ test.job }}" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "test-{{ test.job }}" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"
{% endfor -%}
{% for test in rte.tests -%}
{% for verification in test.verifications %}
.regression_{{ verification.job | replace(from="-", to="_") }}_rules:
  rules:
    - if: $ACTION == "verify-{{ verification.job }}" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "verify-{{ verification.job }}" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"
{% endfor -%}
{% endfor -%}
{% endfor %}
.base: &base
  tags:
    {% for tag in config.ci.tags -%}
    - {{ tag }}
    {%- endfor %}
  cache:
    policy: pull
    key: "${CI_COMMIT_SHA}"
  image: {{ config.ci.image }}
  variables:
    TF_VAR_feature: "sense8"
    TF_VAR_gcp_project_id: $GCP_PROJECT_ID
    TF_VAR_ssh_private_key_file: $KEYS_DIR/$SSH_PRIVATE_KEY_FILE
    TF_VAR_ssh_public_key_file: $KEYS_DIR/$SSH_PUBLIC_KEY_FILE
  before_script:
    - |
      #!/usr/bin/env bash
      cd $CI_PROJECT_DIR/tools/init
      terraform init
      terraform apply -var="f5xc_url=$URL" -var="keys_dir=${KEYS_DIR}" -auto-approve
      P12_FILE=$(terraform output -json | jq -r .data.value.p12_file)
      ENVIRONMENT=$(terraform output -json | jq -r .data.value.environment)
      F5XC_API_URL=$(terraform output -json | jq -r .data.value.api_url)
      F5XC_TENANT=$(terraform output -json | jq -r .data.value.tenant_full)
      F5XC_API_TOKEN_VAR=$(terraform output -json | jq -r .data.value.api_token)
      {% for job_template in config.ci.job_templates -%}
      {% if job_template.name == "base" -%}
      {% for variable in job_template.variables -%}
      export {{ variable.name | upper }}="{{ variable.value }}"
      {% endfor -%}
      {% endif -%}
      {% endfor -%}      
      cd $CI_PROJECT_DIR
      aws s3 cp $SSH_PUBLIC_KEY_FILE_PATH/$SSH_PUBLIC_KEY_FILE $KEYS_DIR
      aws s3 cp $SSH_PRIVATE_KEY_FILE_PATH/$SSH_PRIVATE_KEY_FILE $KEYS_DIR
      aws s3 cp $P12_FILE_PATH/$P12_FILE $KEYS_DIR
      export TF_VAR_environment="$ENVIRONMENT"
      export TF_VAR_f5xc_tenant="$F5XC_TENANT"
      export TF_VAR_f5xc_api_url="$F5XC_API_URL"
      export TF_VAR_f5xc_api_p12_file="${KEYS_DIR}/$P12_FILE"
      export TF_VAR_f5xc_api_token="${!F5XC_API_TOKEN_VAR}"
    - echo $CI_PROJECT_DIR
    - terraform version
{% for rte in rtes -%}
{% for share in rte.shares %}
# {{ share.job | replace(from="_", to="-") }} - deploy
{{ share.job | replace(from="_", to="-") }}-deploy:
  <<: *base
  stage: rte-share-deploy
  rules:
    - !reference [ .deploy_rules, rules ]
    - !reference [ .deploy_rte_rules, rules ]
    - !reference [ .deploy_{{ share.job | replace(from="-", to="_") }}_rules, rules ]
  script:
    - |
      {%- for script in share.scripts %}
      {%- for k, v in script %}
      {%- if k == "apply" %}
      {%- for command in v %}
      {{ command }}
      {%- endfor %}
      {%- endif %}
      {%- endfor %}
      {%- endfor %}
  artifacts:
    paths:
      - $ARTIFACTS_ROOT_DIR/
    expire_in: {{ config.ci.artifacts.expire_in }}
  timeout: {{ rte.ci[share.provider].timeout }}
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure

# {{ share.job | replace(from="_", to="-") }} - artifacts
{{ share.job }}-artifacts:
  <<: *base
  stage: rte-share-artifacts
  rules:
    - !reference [ .destroy_rules, rules ]
    {%- for site in eut.sites %}
    - !reference [ .deploy_{{ site.job | replace(from="-", to="_") }}_rules, rules ]
    - !reference [ .destroy_{{ site.job | replace(from="-", to="_") }}_rules, rules ]
    {%- endfor %}
    {%- for component in rte.components %}
    {%- if component.provider == share.provider and rte.name == share.rte %}
    - !reference [ .deploy_{{ component.job | replace(from="-", to="_") }}_rules, rules ]
    - !reference [ .destroy_{{ component.job | replace(from="-", to="_") }}_rules, rules ]
    {%- endif %}
    {%- endfor %}
  script:
    - |
      {%- for script in share.scripts %}
      {%- for k, v in script %}
      {%- if k == "artifacts" %}
      {%- for command in v %}
      {{ command }}
      {%- endfor %}
      {%- endif %}
      {%- endfor %}
      {%- endfor %}
  artifacts:
    paths:
      - $ARTIFACTS_ROOT_DIR/
    expire_in: {{ config.ci.artifacts.expire_in }}
  timeout: {{ rte.ci[share.provider].timeout }}
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% endfor -%}
{% for component in rte.components %}
# {{ component.job | replace(from="_", to="-") }} - deploy
{{ component.job | replace(from="_", to="-") }}:
  <<: *base
  stage: rte-deploy
  rules:
    - !reference [ .deploy_rules, rules ]
    - !reference [ .deploy_rte_rules, rules ]
    - !reference [ .deploy_{{ component.job | replace(from="-", to="_") }}_rules, rules ]
  script:
      - |
        export TF_VAR_f5xc_api_url_rte=$f5xc_api_url_rte
        export TF_VAR_f5xc_api_token_rte=$f5xc_api_token_rte
        {%- for script in component.scripts %}
        {%- for k, v in script %}
        {%- if k == "apply" %}
        {%- for command in v %}
        {{ command }}
        {%- endfor %}
        {%- endif %}
        {%- endfor %}
        {%- endfor %}
  artifacts:
    paths:
      - $ARTIFACTS_ROOT_DIR/
      {%- for path in rte.ci[component.provider].artifacts.paths %}
      - {{ path }}
      {%- endfor %}
    expire_in: {{ config.ci.artifacts.expire_in }}
  timeout: {{ rte.ci[component.provider].timeout }}
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure

# {{ component.job | replace(from="_", to="-") }} - artifacts
{{ component.job | replace(from="_", to="-") }}-artifacts:
  <<: *base
  rules:
    - !reference [ .regression_test_rules, rules ]
    - !reference [ .destroy_rules, rules ]
    {%- for test in rte.tests %}
    - !reference [ .regression_{{ test.job | replace(from="-", to="_") }}_rules, rules ]
    {%- endfor %}
  stage: rte-artifacts
  script:
      - |
        {%- for script in component.scripts %}
        {%- for k, v in script %}
        {%- if k == "artifacts" %}
        {%- for command in v %}
        {{ command }}
        {%- endfor %}
        {%- endif %}
        {%- endfor %}
        {%- endfor %}
  artifacts:
    paths:
      - $ARTIFACTS_ROOT_DIR/
    expire_in: {{ config.ci.artifacts.expire_in }}
  timeout: 5m
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% endfor -%}
{% endfor -%}
{% for site in eut.sites %}
# eut - {{ site.job }} - deploy
{{ site.job }}-deploy:
  <<: *base
  stage: eut-deploy
  rules:
    - !reference [ .deploy_rules, rules ]
    - !reference [ .deploy_eut_rules, rules ]
    - !reference [ .deploy_{{ site.job | replace(from="-", to="_") }}_rules, rules ]
  script:
      - |
        {%- for script in site.scripts %}
        {%- for k, v in script %}
        {%- if k == "apply" %}
        {%- for command in v %}
        {{ command }}
        {%- endfor %}
        {%- endif %}
        {%- endfor %}
        {%- endfor %}
  artifacts:
    paths:
      - $ARTIFACTS_ROOT_DIR/
    expire_in: {{ config.ci.artifacts.expire_in }}
  timeout: {{ eut.module.ci.timeout }}
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure

# eut - {{ site.job }} - artifacts
{{ site.job }}-artifacts:
  <<: *base
  stage: eut-artifacts
  rules:
    - !reference [ .deploy_rte_rules, rules ]
    - !reference [ .destroy_rules, rules ]
    - !reference [ .destroy_rte_rules, rules ]
    {%- for rte in rtes %}
    {%- for test in rte.tests %}
    - !reference [ .regression_{{ test.job | replace(from="-", to="_") }}_rules, rules ]
    {%- endfor %}
    {%- endfor %}
  script:
      - |
        {%- for script in site.scripts %}
        {%- for k, v in script %}
        {%- if k == "artifacts" %}
        {%- for command in v %}
        {{ command }}
        {%- endfor %}
        {%- endif %}
        {%- endfor %}
        {%- endfor %}
  artifacts:
    paths:
      - $ARTIFACTS_ROOT_DIR/
    expire_in: {{ config.ci.artifacts.expire_in }}
  timeout: {{ eut.module.ci.timeout }}
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% endfor -%}
{% for feature in features %}
# feature - {{ feature.job }} - deploy
{{ feature.job }}-deploy:
  <<: *base
  stage: feature-deploy
  rules:
    - !reference [ .deploy_rules, rules ]
    - !reference [ .deploy_feature_rules, rules ]
    - !reference [ .deploy_{{ feature.job | replace(from="-", to="_") }}_rules, rules ]
  script:
      - |
        {%- for script in feature.scripts %}
        {%- for k, v in script %}
        {%- if k == "apply" %}
        {%- for command in v %}
        {{ command }}
        {%- endfor %}
        {%- endif %}
        {%- endfor %}
        {%- endfor %}
  artifacts:
    paths:
      - $ARTIFACTS_ROOT_DIR/
    expire_in: 3h
  timeout: {{ feature.ci.timeout }}
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure

# feature - {{ feature.job }} - artifacts
{{ feature.job }}-artifacts:
  <<: *base
  stage: feature-artifacts
  rules:
    {%- for site in eut.sites %}
    - !reference [ .deploy_{{ site.job | replace(from="-", to="_") }}_rules, rules ]
    {%- endfor %}
    {%- for rte in rtes %}
    {%- for test in rte.tests %}
    - !reference [ .regression_{{ test.job | replace(from="-", to="_") }}_rules, rules ]
    {%- endfor %}
    {%- endfor %}
  script:
      - |
        {%- for script in feature.scripts %}
        {%- for k, v in script %}
        {%- if k == "artifacts" %}
        {%- for command in v %}
        {{ command }}
        {%- endfor %}
        {%- endif %}
        {%- endfor %}
        {%- endfor %}
  artifacts:
    paths:
      - $ARTIFACTS_ROOT_DIR/
    expire_in: 3h
  timeout: {{ feature.ci.timeout }}
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% endfor -%}
{% for rte in rtes -%}
{% for test in rte.tests %}
# test - {{ test.job }} - deploy
{{ test.job }}-deploy:
  <<: *base
  rules:
    - !reference [ .regression_test_rules, rules ]
    - !reference [ .regression_{{ test.job | replace(from="-", to="_") }}_rules, rules ]
  stage: regression-test
  script:
      - |
        {%- for script in test.scripts %}
        {%- for k, v in script %}
        {%- if k == "apply" %}
        {%- for command in v %}
        {{ command }}
        {%- endfor %}
        {%- endif %}
        {%- endfor %}
        {%- endfor %}
  artifacts:
    paths:
      - $ARTIFACTS_ROOT_DIR/
    expire_in: {{ config.ci.artifacts.expire_in }}
  timeout: {{ test.ci.timeout }}
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure

# test - {{ test.job }} - artifacts
{{ test.job }}-artifacts:
  <<: *base
  rules:
    - !reference [ .regression_verification_rules, rules ]
    {%- for verification in test.verifications %}
    - !reference [ .regression_{{ verification.job | replace(from="-", to="_") }}_rules, rules ]
    {%- endfor %}
  stage: regression-test-artifacts
  script:
      - |
        {%- for script in test.scripts %}
        {%- for k, v in script %}
        {%- if k == "artifacts" %}
        {%- for command in v %}
        {{ command }}
        {%- endfor %}
        {%- endif %}
        {%- endfor %}
        {%- endfor %}
  artifacts:
    paths:
      - $ARTIFACTS_ROOT_DIR/
    expire_in: {{ config.ci.artifacts.expire_in }}
  timeout: {{ test.ci.timeout }}
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% endfor -%}
{% endfor -%}
{% for rte in rtes -%}
{% for test in rte.tests -%}
{% for verification in test.verifications %}
# verification - {{ verification.job }} - deploy
{{ verification.job }}-deploy:
  <<: *base
  rules:
    - !reference [ .regression_verification_rules, rules ]
    - !reference [ .regression_{{ verification.job | replace(from="-", to="_") }}_rules, rules ]
  stage: regression-test-verify
  script:
      - |
        {%- for script in verification.scripts %}
        {%- for k, v in script %}
        {%- if k == "apply" %}
        {%- for command in v %}
        {{ command }}
        {%- endfor %}
        {%- endif %}
        {%- endfor %}
        {%- endfor %}
  timeout: {{ verification.ci.timeout }}
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% endfor -%}
{% endfor -%}
{% endfor -%}

{% for feature in features %}
# feature - {{ feature.job }} - destroy
{{ feature.job }}-destroy:
  <<: *base
  stage: feature-destroy
  rules:
    - !reference [ .destroy_rules, rules ]
    - !reference [ .destroy_feature_rules, rules ]
    - !reference [ .destroy_{{ feature.job | replace(from="-", to="_") }}_rules, rules ]
  script:
      - |
        {%- for script in feature.scripts %}
        {%- for k, v in script %}
        {%- if k == "destroy" %}
        {%- for command in v %}
        {{ command }}
        {%- endfor %}
        {%- endif %}
        {%- endfor %}
        {%- endfor %}
  timeout: {{ feature.ci.timeout }}
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% endfor -%}
{% for site in eut.sites %}
# eut {{ site.job }} - destroy
{{ site.job }}-destroy:
  <<: *base
  stage: eut-destroy
  rules:
    - !reference [ .destroy_rules, rules ]
    - !reference [ .destroy_eut_rules, rules ]
    - !reference [ .destroy_{{ site.job | replace(from="-", to="_") }}_rules, rules ]
  script:
      - |
        {%- for script in site.scripts %}
        {%- for k, v in script %}
        {%- if k == "destroy" %}
        {%- for command in v %}
        {{ command }}
        {%- endfor %}
        {%- endif %}
        {%- endfor %}
        {%- endfor %}
  artifacts:
    paths:
      - $ARTIFACTS_ROOT_DIR/
    expire_in: {{ config.ci.artifacts.expire_in }}
  timeout: {{ eut.module.ci.timeout }}
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% endfor -%}
{% for rte in rtes -%}
{% for component in rte.components %}
# {{ component.job | replace(from="_", to="-") }} - destroy
{{ component.job | replace(from="_", to="-") }}-destroy:
  <<: *base
  stage: rte-destroy
  rules:
    - !reference [ .destroy_rules, rules ]
    - !reference [ .destroy_rte_rules, rules ]
    - !reference [ .destroy_{{ component.job | replace(from="-", to="_") }}_rules, rules ]
  script:
    - |
      export TF_VAR_f5xc_api_url_rte=$f5xc_api_url_rte
      export TF_VAR_f5xc_api_token_rte=$f5xc_api_token_rte
      {%- for script in component.scripts %}
      {%- for k, v in script %}
      {%- if k == "destroy" %}
      {%- for command in v %}
      {{ command }}
      {%- endfor %}
      {%- endif %}
      {%- endfor %}
      {%- endfor %}
  {%- if rte.ci[component.provider].artifacts.needs_deploy %}
  needs:
    - project: $CI_PROJECT_PATH
      job: {{ component.job | replace(from="_", to="-") }}
      ref: $CI_COMMIT_REF_NAME
      artifacts: true
  {%- endif %}
  timeout: {{ rte.ci[component.provider].timeout }}
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% endfor -%}
{% for share in rte.shares %}
# {{ share.job | replace(from="_", to="-") }} - destroy
{{ share.job | replace(from="_", to="-") }}-destroy:
  <<: *base
  stage: rte-share-destroy
  rules:
    - !reference [ .destroy_rules, rules ]
    - !reference [ .destroy_rte_rules, rules ]
    - !reference [ .destroy_{{ share.job | replace(from="-", to="_") }}_rules, rules ]
  script:
    - |
      {%- for script in share.scripts %}
      {%- for k, v in script %}
      {%- if k == "destroy" %}
      {%- for command in v %}
      {{ command }}
      {%- endfor %}
      {%- endif %}
      {%- endfor %}
      {%- endfor %}
  timeout: {{ rte.ci[share.provider].timeout }}
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% endfor -%}
{% endfor -%}