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
  FEATURE_{{ feature.base.name | upper }}_ROOT_TF_VAR_FILE: "$FEATURE_ROOT_DIR/{{ feature.base.name }}/terraform.tfvars"
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

.regression_test_rules:
  rules:
    - if: $ACTION == "test" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "test" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.regression_verification_rules:
  rules:
    - if: $ACTION == "verify" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "verify" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

{% for rte in rtes -%}
{% for test in rte.tests %}
.regression_test_{{ test.rte }}_{{ test.name | replace(from="-", to="_") }}:
  rules:
    - if: $ACTION == "test-{{ test.rte | replace(from="_", to="-") }}-{{ test.name | replace(from="_", to="-") }}" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "test-{{ test.rte | replace(from="_", to="-") }}-{{ test.name | replace(from="_", to="-") }}" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"
{% endfor -%}
{% endfor -%}
{% for rte in rtes -%}
{% for test in rte.tests -%}
{%for verification in test.verifications %}
.regression_verification_{{ test.rte }}_{{ test.name | replace(from="-", to="_") }}_{{ verification.name | replace(from="-", to="_") }}:
  rules:
    - if: $ACTION == "verification-{{ test.rte | replace(from="_", to="-") }}-{{ test.name | replace(from="_", to="-") }}-{{ verification.name | replace(from="_", to="-") }}" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "verification-{{ test.rte | replace(from="_", to="-") }}-{{ test.name | replace(from="_", to="-") }}-{{ verification.name | replace(from="_", to="-") }}" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"
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
      if [ "$ENVIRONMENT" == "production" ]; then
        export TF_VAR_f5xc_api_token=$PRODUCTION_API_TOKEN  
      elif [ "$ENVIRONMENT" == "staging" ]; then
        export TF_VAR_f5xc_api_token=$STAGING_API_TOKEN
      fi
    - terraform version
    - echo $CI_PROJECT_DIR
    - cd $CI_PROJECT_DIR

# eut - apply
eut-apply:
  <<: *base
  stage: eut-apply
  rules:
    - !reference [ .deploy_rules, rules ]
    - !reference [ .deploy_eut_rules, rules ]
  script:
      - |
        #!/usr/bin/env bash
        {% for provider in eut.provider -%}
        cd $EUT_ROOT_DIR/{{ eut.base.module }}/{{ provider }}
        terraform init --backend-config="key=$S3_EUT_ROOT/{{ project.name }}/{{ eut.base.module }}/{{ provider }}/{{ eut.module.release }}"
        {% if provider == "azure" -%}
        # terraform import -var-file=$EUT_ROOT_TF_VAR_FILE -var-file=$S3_EUT_ROOT/{{ project.name }}/{{ eut.base.module }}/{{ provider }}/terraform.tfvars.json -var-file=$RTE_SHARED_ARTIFACTS_FILE azurerm_marketplace_agreement.xc /subscriptions/$ARM_SUBSCRIPTION_ID/providers/Microsoft.MarketplaceOrdering/agreements/volterraedgeservices/offers/entcloud_voltmesh_voltstack_node/plans/freeplan_entcloud_voltmesh_voltstack_node
        {% endif -%}  
        terraform apply -var-file=$EUT_ROOT_TF_VAR_FILE -var-file=$S3_EUT_ROOT/{{ project.name }}/{{ eut.base.module }}/{{ provider }}/terraform.tfvars.json -var-file=$RTE_SHARED_ARTIFACTS_FILE -auto-approve
        terraform output > $EUT_ROOT_DIR/{{ eut.base.module }}/{{ provider }}/site.tfvars
        {% endfor %}
  timeout: {{ eut.module.ci.timeout }}
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% for rte in rtes -%}
{% for connection in rte.connections %}
# {{ connection.job | replace(from="_", to="-") }} - apply
{{ connection.job | replace(from="_", to="-") }}-apply:
  <<: *base
  stage: rte-apply
  rules:
    - !reference [ .deploy_rules, rules ]
    - !reference [ .deploy_rte_rules, rules ]
  script:
      - |
        {% for script in connection.scripts -%}
        {% for k, v in script -%}
        {% if k == "apply" -%}
        {% for command in v -%}
        {{ command }}
        {% endfor -%}
        {% endif -%}
        {% endfor -%}
        {% endfor %}
  artifacts:
    paths:
      - $ARTIFACTS_ROOT_DIR/
    expire_in: {{ config.ci.artifacts.expire_in }}
  timeout: {{ rte.ci[connection.provider].timeout }}
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% endfor -%}
{% endfor %}
{% for feature in features %}
# feature - {{ eut.base.module }} - {{ feature.base.name }} - apply
feature-{{ eut.base.module }}-{{ feature.base.name }}-apply:
  <<: *base
  stage: feature-apply
  rules:
    - !reference [ .deploy_rules, rules ]
    - !reference [ .deploy_feature_rules, rules ]
  script:
      - |
        #!/usr/bin/env bash
        cd $FEATURES_ROOT_DIR/{{ feature.base.name }}
        terraform init --backend-config="key=$S3_EUT_ROOT/{{ eut.base.module }}/features/{{ feature.base.name }}/{{ feature.module.release }}"  
        terraform apply -var-file=$EUT_ROOT_TF_VAR_FILE -auto-approve
        terraform output > $FEATURES_ROOT_DIR/{{ feature.base.name }}/feature.tfvars
  timeout:
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% endfor -%}
{% for rte in rtes -%}
{% for test in rte.tests %}
# test - {{ test.job }} - apply
{{ test.job }}-apply:
  <<: *base
  rules:
    - !reference [ .regression_test_rules, rules ]
    - !reference [ .regression_test_{{ test.rte }}_{{ test.name | replace(from="-", to="_") }}, rules ]
  stage: regression-test
  script:
      - |
        #!/usr/bin/env bash
        cd $CI_PROJECT_DIR/{{ config.tests.path }}/{{ test.module }}
        terraform init --backend-config="key=$S3_TESTS_ROOT/{{ test.module }}/"
        terraform apply -compact-warnings -var-file=$ARTIFACTS_ROOT_DIR/{{ test.module }}/artifacts.tfvars -auto-approve
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
# verification - {{ verification.job }} - apply
{{ verification.job }}-apply:
  <<: *base
  rules:
    - !reference [ .regression_verification_rules, rules ]
    - !reference [ .regression_verification_{{ test.rte }}_{{ test.name | replace(from="-", to="_") }}_{{ verification.name | replace(from="-", to="_") }}, rules ]
  stage: regression-test-verify
  script:
      - |
        #!/usr/bin/env bash
        cd $CI_PROJECT_DIR/{{ config.verifications.path }}/{{ verification.module }}
        terraform init --backend-config="key=$S3_VERIFICATIONS_ROOT/{{ verification.module }}/"
        terraform apply -compact-warnings -var-file=$ARTIFACTS_ROOT_DIR/{{ verification.module }}/artifacts.tfvars -auto-approve
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