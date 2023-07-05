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

.regression_test_rules:
  rules:
    - if: $ACTION == "test" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "test" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

regression_test_verify_rules:
  rules:
    - if: $ACTION == "verify" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "verify" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

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
        cd $EUT_ROOT_DIR/{{ eut.name }}/{{ provider }}
        terraform init --backend-config="key=$S3_EUT_ROOT/{{ project.name }}/{{ eut.name }}/{{ provider }}"
        {% if provider == "azure" -%}
        # terraform import -var-file=$EUT_ROOT_TF_VAR_FILE -var-file=$S3_EUT_ROOT/{{ project.name }}/{{ eut.name }}/{{ provider }}/terraform.tfvars.json -var-file=$RTE_SHARED_ARTIFACTS_FILE azurerm_marketplace_agreement.xc /subscriptions/$ARM_SUBSCRIPTION_ID/providers/Microsoft.MarketplaceOrdering/agreements/volterraedgeservices/offers/entcloud_voltmesh_voltstack_node/plans/freeplan_entcloud_voltmesh_voltstack_node
        {% endif -%}  
        terraform apply -var-file=$EUT_ROOT_TF_VAR_FILE -var-file=$S3_EUT_ROOT/{{ project.name }}/{{ eut.name }}/{{ provider }}/terraform.tfvars.json -var-file=$RTE_SHARED_ARTIFACTS_FILE -auto-approve
        terraform output > $EUT_ROOT_DIR/{{ eut.name }}/{{ provider }}/site.tfvars
        {% endfor %}
  timeout: {{ eut.ci.timeout }}
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% for feature in features %}
# feature - {{ eut.name }} - {{ feature.name }} - apply
feature-{{ eut.name }}-{{ feature.name }}-apply:
  <<: *base
  stage: feature-apply
  rules:
    - !reference [ .deploy_rules, rules ]
    - !reference [ .deploy_features_rules, rules ]
  script:
      - |
        #!/usr/bin/env bash
        cd $FEATURES_ROOT_DIR/{{ feature.name }}
        terraform init --backend-config="key=$S3_EUT_ROOT/{{ eut.name }}/features/{{ feature.name }}"  
        terraform apply -var-file=$EUT_ROOT_TF_VAR_FILE -auto-approve
        terraform output > $FEATURES_ROOT_DIR/{{ feature.name }}/feature.tfvars
  timeout: {{ feature.ci.timeout }}
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% endfor -%}
{% for provider in eut.provider -%}
{% for rte in rtes %}
# rte - {{ provider }} - {{ rte.cfg.module | replace(from="_", to="-")}} - apply
rte-{{ provider }}-{{ rte.cfg.module | replace(from="_", to="-")}}-apply:
  <<: *base
  stage: rte-apply
  rules:
    - !reference [ .deploy_rules, rules ]
    - !reference [ .deploy_rte_rules, rules ]
  script:
      - |
  artifacts:
    paths:
      - $ARTIFACTS_ROOT_DIR/
    expire_in: {{ config.ci.artifacts.expire_in }}
  timeout: {{ rte.module.ci.timeout }}
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% endfor -%}
{% endfor -%}
{% for provider in eut.provider -%}
{% for rte in rtes -s%}
{% for test in rte.cfg.tests %}
# test - {{ test.module | replace(from="_", to="-")}} - {{ rte.cfg.module | replace(from="_", to="-") }} - {{ provider }} - apply
regression-test-{{ provider }}-{{ rte.cfg.module | replace(from="_", to="-") }}-{{ test.name }}:
  <<: *base
  rules:
    - !reference [ .regression_test_rules, rules ]
    - !reference [ .regression_test_{{ test.name | replace(from="-", to="_") }}, rules ]
  stage: regression-test
  script:
      - |
        #!/usr/bin/env bash
        cd $CI_PROJECT_DIR/{{ config.tests.path }}/{{ test.module }}
        terraform init --backend-config="key=$S3_TESTS_ROOT/{{ test.module }}/{{ provider }}"
        terraform apply -compact-warnings -var-file=$ARTIFACTS_ROOT_DIR/{{ test.module }}/{{ provider }}/artifacts.tfvars -auto-approve
  timeout: {{ test.ci.timeout }}
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% endfor -%}
{% endfor -%}
{% endfor %}









{{ project.name }}
{{ eut.name }}
{% for rte in rtes %}
{{ rte.module }}
{% endfor %}
{% for rte in rtes %}
{% for test in rte.cfg.tests %}
{{ test.module }}
{% endfor %}
{% endfor %}

{% for rte in rtes %}
{% for test in rte.cfg.tests %}
{% for verification in test.verifications %}
{{ verification.module }}
{% endfor %}
{% endfor %}
{% endfor %}