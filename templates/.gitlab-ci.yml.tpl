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
{% for rte in rtes -%}
{% for share in rte.shares %}
.deploy_{{ share.job | replace(from="-", to="_") }}_rules:
  rules:
    - if: $ACTION == "deploy-{{ share.job | replace(from="_", to="-") }}" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "deploy-{{ share.job | replace(from="_", to="-") }}" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.destroy_{{ share.job | replace(from="-", to="_") }}_rules:
  rules:
    - if: $ACTION == "destroy-{{ share.job | replace(from="_", to="-") }}" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "destroy-{{ share.job | replace(from="_", to="-") }}" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"
{% endfor -%}
{% for component in rte.components %}
.deploy_{{ component.job | replace(from="-", to="_") }}_rules:
  rules:
    - if: $ACTION == "deploy-{{ component.job | replace(from="_", to="-") }}" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "deploy-{{ component.job | replace(from="_", to="-") }}" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.destroy_{{ component.job | replace(from="-", to="_") }}_rules:
  rules:
    - if: $ACTION == "destroy-{{ component.job | replace(from="_", to="-") }}" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "destroy-{{ component.job | replace(from="_", to="-") }}" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"
{% endfor -%}
{% for test in rte.tests %}
.regression_test_{{ test.rte }}_{{ test.name | replace(from="-", to="_") }}:
  rules:
    - if: $ACTION == "test-{{ test.rte | replace(from="_", to="-") }}-{{ test.name | replace(from="_", to="-") }}" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "test-{{ test.rte | replace(from="_", to="-") }}-{{ test.name | replace(from="_", to="-") }}" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"
{% endfor -%}
{% for test in rte.tests -%}
{%for verification in test.verifications %}
.regression_verification_{{ test.rte }}_{{ test.name | replace(from="-", to="_") }}_{{ verification.name | replace(from="-", to="_") }}:
  rules:
    - if: $ACTION == "verify-{{ test.rte | replace(from="_", to="-") }}-{{ test.name | replace(from="_", to="-") }}-{{ verification.name | replace(from="_", to="-") }}" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "verify-{{ test.rte | replace(from="_", to="-") }}-{{ test.name | replace(from="_", to="-") }}-{{ verification.name | replace(from="_", to="-") }}" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"
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
    TF_VAR_feature: $FEATURE
    TF_VAR_environment: $ENVIRONMENT
    TF_VAR_gcp_project_id: $GCP_PROJECT_ID
    TF_VAR_ssh_private_key_file: $KEYS_DIR/$SSH_PRIVATE_KEY_FILE
    TF_VAR_ssh_public_key_file: $KEYS_DIR/$SSH_PUBLIC_KEY_FILE
  before_script:
    - |
      #!/usr/bin/env bash
      aws s3 cp $SSH_PUBLIC_KEY_FILE_PATH/$SSH_PUBLIC_KEY_FILE $KEYS_DIR
      aws s3 cp $SSH_PRIVATE_KEY_FILE_PATH/$SSH_PRIVATE_KEY_FILE $KEYS_DIR
      aws s3 cp $P12_FILE_PATH/$P12_FILE $KEYS_DIR
      export TF_VAR_f5xc_api_p12_file="${KEYS_DIR}/${P12_FILE}"
      export TF_VAR_f5xc_api_token="${!F5XC_API_TOKEN}"
    - terraform version
    - echo $CI_PROJECT_DIR
    - cd $CI_PROJECT_DIR
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
{{ share.job | replace(from="_", to="-") }}-artifacts:
  <<: *base
  stage: rte-share-artifacts
  rules:
    - !reference [ .deploy_rules, rules ]
    - !reference [ .deploy_rte_rules, rules ]
    {%- for component in rte.components %}
    {%- if component.provider == share.provider and rte.name == share.rte %}
    - !reference [ .deploy_{{ component.job | replace(from="-", to="_") }}_rules, rules ]
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
    - !reference [ .regression_test_{{ component.rte }}_{{ test.name | replace(from="-", to="_") }}, rules ]
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
# eut {{ eut.module.name }} - {{ site.name | replace(from="_", to="-") }} - deploy
eut-{{ eut.module.name }}-{{ site.name | replace(from="_", to="-") }}-deploy:
  <<: *base
  stage: eut-deploy
  rules:
    - !reference [ .deploy_rules, rules ]
    - !reference [ .deploy_eut_rules, rules ]
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

# eut {{ eut.module.name }} - {{ site.name | replace(from="_", to="-") }} - artifacts
eut-{{ eut.module.name }}-{{ site.name | replace(from="_", to="-") }}-artifacts:
  <<: *base
  stage: eut-artifacts
  rules:
    - !reference [ .deploy_rte_rules, rules ]
    - !reference [ .destroy_rules, rules ]
    - !reference [ .destroy_rte_rules, rules ]
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
{% endfor %}
{% for feature in features %}
# {{ feature.job }} - deploy
{{ feature.job }}-deploy:
  <<: *base
  stage: feature-deploy
  rules:
    - !reference [ .deploy_rules, rules ]
    - !reference [ .deploy_feature_rules, rules ]
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
    - !reference [ .regression_test_{{ test.rte }}_{{ test.name | replace(from="-", to="_") }}, rules ]
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
    - !reference [ .regression_verification_{{ test.rte }}_{{ test.name | replace(from="-", to="_") }}_{{ verification.name | replace(from="-", to="_") }}, rules ]
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
    - !reference [ .regression_verification_{{ test.rte }}_{{ test.name | replace(from="-", to="_") }}_{{ verification.name | replace(from="-", to="_") }}, rules ]
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
# feature - {{ eut.base.module }} - {{ feature.name }} - destroy
feature-{{ eut.base.module }}-{{ feature.name }}-destroy:
  <<: *base
  stage: feature-destroy
  rules:
    - !reference [ .destroy_rules, rules ]
    - !reference [ .destroy_feature_rules, rules ]
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
# eut {{ eut.module.name }} - {{ site.name | replace(from="_", to="-") }} - destroy
eut-{{ eut.module.name }}-{{ site.name | replace(from="_", to="-") }}-destroy:
  <<: *base
  stage: eut-destroy
  rules:
    - !reference [ .destroy_rules, rules ]
    - !reference [ .destroy_eut_rules, rules ]
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
  script:
    - |
      {%- for script in component.scripts %}
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
{% endfor -%}