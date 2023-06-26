stages:
  {% for stage in ci.stages -%}
  - {{ stage }}
  {% endfor %}
variables:
  {% for variable in rc.ci.variables -%}
  {{ variable.name | upper }}: "{{ variable.value }}"
  {% endfor -%}
  {% for provider, tmvc in providers -%}
  {% for rte in tmvc.rtes -%}
  RTE_{{ rte.name | upper }}_{{ provider | upper }}_ROOT_DIR: "$RTE_ROOT_DIR/{{ rte.name }}/{{ provider }}"
  RTE_{{ rte.name | upper }}_{{ provider | upper }}_ROOT_TF_VAR_FILE:  "$RTE_ROOT_DIR/{{ rte.name }}/terraform.tfvars.json"
  RTE_{{ rte.name | upper }}_{{ provider | upper }}_TF_VAR_FILE:  "$RTE_ROOT_DIR/{{ rte.name }}/{{ provider }}/terraform.tfvars.json"
  RTE_{{ rte.name | upper }}_{{ provider | upper }}_COMMON_ARTIFACTS_FILE: "$ARTIFACTS_ROOT_DIR/rte_{{ rte.name }}_common.tfvars.json"
  RTE_{{ rte.name | upper }}_{{ provider | upper }}_ARTIFACTS_FILE: "$ARTIFACTS_ROOT_DIR/{{ rte.name }}/{{ provider }}/artifacts.tfvars"
  {% endfor -%}
  {% endfor %}
.deploy_rules:
  rules:
    - if: $ACTION == "deploy" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "deploy" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.destroy_rules:
  rules:
    - if: $ACTION == "destroy" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "destroy" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.regression_test_rules:
  rules:
    - if: $ACTION == "test" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "test" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"

.regression_test_verify_rules:
  rules:
    - if: $ACTION == "verify" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "verify" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"
{% for provider, values in providers -%}
{% for tm in values.tmvc %}
.regression_test_{{ tm.name | replace(from="-", to="_") }}:
  rules:
    - if: $ACTION == "test_{{ tm.name | replace(from="-", to="_") }}" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "test_{{ tm.name | replace(from="-", to="_") }}" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"
{% endfor -%}
{% endfor -%}
{% for provider, values in providers -%}
{% for tm in values.tmvc -%}
{% for verification in tm.verifications %}
.regression_test_verify_{{ tm.name | replace(from="-", to="_") }}_{{ verification.name | replace(from="-", to="_") }}:
  rules:
    - if: $ACTION == "test_verify_{{ tm.name | replace(from="-", to="_") }}_{{ verification.name | replace(from="-", to="_") }}" && $CI_PIPELINE_SOURCE == "trigger" && $CI_PIPELINE_TRIGGERED == "true"
    - if: $ACTION == "test_verify_{{ tm.name | replace(from="-", to="_") }}_{{ verification.name | replace(from="-", to="_") }}" && $CI_PIPELINE_SOURCE == "web" && $CI_PIPELINE_TRIGGERED == "true"
{% endfor -%}
{% endfor -%}
{% endfor %}
.base: &base
  tags:
    {% for tag in rc.ci.tags -%}
    - {{ tag }}
    {% endfor %}
  cache:
    policy: pull
    key: "${CI_COMMIT_SHA}"
  image: {{ rc.ci.image }}
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
{% for provider, values in providers -%}
{% for rte in values.rtes %}
# rte - {{ provider }} - {{ rte.name | replace(from="_", to="-")}} - artifacts
rte-{{ provider }}-{{ rte.name | replace(from="_", to="-")}}-artifacts:
  <<: *base
  rules:
    - !reference [ .regression_test_rules, rules ]
    - !reference [ .destroy_rules, rules ]
    {%- for test in values.tmvc %}
    - !reference [ .regression_test_{{ test.name | replace(from="-", to="_") }}, rules ]
    {%- endfor %}
  stage: rte-artifacts
  script:
      {% for script in rte.scripts -%}
      {% if script.name == 'artifacts' -%}
      {% for line in script.value -%}
      {{ line }}
      {% endfor -%}
      {% endif -%}
      {% endfor -%}
      - |
        #!/usr/bin/env bash
        cd $RTE_{{ rte.name | upper }}_{{ provider | upper }}_ROOT_DIR
        terraform init --backend-config="key=features/$FEATURE/$ENVIRONMENT/{{ rc.eut.path }}/{{ rc.rte.path }}/{{ rte.name }}/{{ provider }}"
        terraform output > $RTE_{{ rte.name | upper }}_{{ provider | upper }}_ARTIFACTS_FILE
  artifacts:
    paths:
      - $ARTIFACTS_ROOT_DIR/{{ provider }}/{{ rte.name }}/artifacts.tfvars
    expire_in: {{ rc.ci.artifacts.expire_in }}
  timeout: 5m
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure

# rte - {{ provider }} - {{ rte.name | replace(from="_", to="-")}} - apply
rte-{{ provider }}-{{ rte.name | replace(from="_", to="-")}}-apply:
  <<: *base
  stage: rte-apply
  rules:
    - !reference [ .deploy_rules, rules ]
  script:
      {% for script in rte.scripts -%}
      {% if script.name == 'apply' -%}
      {% for line in script.value -%}
      {{ line }}
      {% endfor -%}
      {% endif -%}
      {% endfor -%}
      - |
        #!/usr/bin/env bash
        cd $RTE_{{ rte.name | upper }}_{{ provider | upper }}_ROOT_DIR
        terraform init --backend-config="key=features/$FEATURE/$ENVIRONMENT/{{ rc.eut.path }}/{{ rc.rte.path }}/{{ rte.name }}/{{ provider }}"
        terraform apply -var-file=$RTE_{{ rte.name | upper }}_{{ provider | upper }}_ROOT_TF_VAR_FILE -var-file=$RTE_{{ rte.name | upper }}_{{ provider | upper }}_TF_VAR_FILE -auto-approve
        mkdir -p $ARTIFACTS_ROOT_DIR/{{ rte.name }}/{{ provider }}
        terraform output > $RTE_{{ rte.name | upper }}_{{ provider | upper }}_ARTIFACTS_FILE
        echo "{{ provider }}_destination_ip=$(terraform output destination_ip)" >> $RTE_{{ rte.name | upper }}_{{ provider | upper }}_COMMON_ARTIFACTS_FILE
  artifacts:
    paths:
      - $ARTIFACTS_ROOT_DIR/
    expire_in: {{ rc.ci.artifacts.expire_in }}
  timeout: 30m
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% endfor -%}
{% endfor %}
# eut - apply
eut-apply:
  <<: *base
  stage: eut-apply
  rules:
    - !reference [ .deploy_rules, rules ]
  script:
      - |
        #!/usr/bin/env bash
        {% for provider, values in providers -%}
        cd $EUT_ROOT_DIR/{{ provider }}
        {% if provider == "azure" -%}
        terraform import azurerm_marketplace_agreement.xc /subscriptions/e9cbbd48-704d-4dfa-bf62-60edda755a66/providers/Microsoft.MarketplaceOrdering/agreements/volterraedgeservices/offers/volterra-node/plans/volterra-node        
        {% endif -%}
        terraform init --backend-config="key=features/$FEATURE/$ENVIRONMENT/{{ rc.eut.path }}/provider/{{ provider }}"
        terraform apply -var-file=$EUT_ROOT_TF_VAR_FILE -var-file=$EUT_ROOT_DIR/{{ provider }}/terraform.tfvars.json {% for rte in values.rtes -%}-var-file=$RTE_{{ rte.name | upper }}_{{ provider | upper }}_ARTIFACTS_FILE {% endfor -%} -auto-approve
        terraform output > $EUT_ROOT_DIR/{{ provider }}/site.tfvars
        {% endfor -%}
        cd $EUT_ROOT_DIR/common
        terraform init --backend-config="key=features/$FEATURE/$ENVIRONMENT/{{ rc.eut.path }}/common"
        terraform apply -var-file=$EUT_ROOT_TF_VAR_FILE -var-file=$EUT_TF_VAR_FILE {% for provider, values in providers %}-var-file=$EUT_ROOT_DIR/{{ provider }}/site.tfvars {% endfor %} {% for provider, values in providers %}{% for rte in values.rtes %}-var-file=$RTE_{{ rte.name | upper }}_{{ provider | upper }}_COMMON_ARTIFACTS_FILE {% endfor %}{% endfor %}-auto-approve
  timeout: 1h 30m
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% for provider, values in providers -%}
{% for test in values.tmvc %}
# test - {{ test.module | replace(from="_", to="-")}} - {{ test.rte.name  | replace(from="_", to="-") }} - {{ provider }} - apply
regression-test-{{ test.name }}:
  <<: *base
  rules:
    - !reference [ .regression_test_rules, rules ]
    - !reference [ .regression_test_{{ test.name | replace(from="-", to="_") }}, rules ]
  stage: regression-test
  script:
      - |
        #!/usr/bin/env bash
        cd $CI_PROJECT_DIR/{{ rc.tests.path }}/{{ test.name }}
        terraform init --backend-config="key=features/$FEATURE/$ENVIRONMENT/{{ rc.eut.path }}/{{ rc.tests.path }}/{{ test.name }}/{{ test.rte.name }}/{{ provider }}"
        terraform apply -compact-warnings -var-file=$ARTIFACTS_ROOT_DIR/{{ provider }}/{{ test.rte.name }}/artifacts.tfvars -auto-approve
  timeout: 30m
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% endfor -%}
{% endfor -%}
{% for provider, values in providers -%}
{% for test in values.tmvc -%}
{% for verification in test.verifications %}
# verify {{ test.module | replace(from="_", to="-")}} - {{ test.rte.name  | replace(from="_", to="-") }} - {{ verification.name | replace(from="_", to="-") }} - {{ provider }} - apply
regression-test-verify-{{ test.name }}-{{ verification.name }}:
  <<: *base
  rules:
    - !reference [ .regression_test_verify_rules, rules ]
    - !reference [ .regression_test_verify_{{ test.name | replace(from="-", to="_") }}_{{ verification.name | replace(from="-", to="_") }}, rules ]
  stage: regression-test
  script:
      - |
        #!/usr/bin/env bash
        cd $CI_PROJECT_DIR/{{ rc.verifications.path }}/{{ test.name }}
        terraform init --backend-config="key=features/$FEATURE/$ENVIRONMENT/{{ rc.eut.path }}{{ rc.verifications.path }}/{{ test.rte.name }}/{{ verification.name }}"
        terraform apply -compact-warnings -var-file=$ARTIFACTS_ROOT_DIR/{{ provider }}/{{ test.rte.name }}/artifacts.tfvars -auto-approve
  timeout: 30m
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% endfor -%}
{% endfor -%}
{% endfor %}
# eut - destroy
eut-destroy:
  <<: *base
  stage: eut-destroy
  rules:
    - !reference [ .destroy_rules, rules ]
  script:
      - |
         #!/usr/bin/env bash
         cd $EUT_ROOT_DIR/common
         terraform init --backend-config="key=features/$FEATURE/$ENVIRONMENT/sites/common"
         terraform destroy -var-file=$EUT_ROOT_TF_VAR_FILE -var-file=$EUT_TF_VAR_FILE {% for provider, values in providers -%}-var-file=$EUT_ROOT_DIR/{{ provider }}/site.tfvars {% endfor -%} {% for provider, values in providers -%}{% for rte in values.rtes -%}-var-file=$RTE_{{ provider | upper }}_{{ rte.name | upper }}_COMMON_TF_VAR_FILE {% endfor -%} {% endfor -%} -auto-approve
         {% for provider, values in providers -%}
         cd $EUT_ROOT_DIR/{{ provider }}
         terraform init --backend-config="key=features/$FEATURE/$ENVIRONMENT/sites/{{ provider }}" 
         terraform destroy -var-file=$EUT_ROOT_TF_VAR_FILE -var-file=$EUT_ROOT_DIR/{{ provider }}/terraform.tfvars.json {% for rte in values.rtes -%}-var-file=$RTE_{{ provider | upper }}_{{ rte.name | upper }}_ARTIFACTS_FILE {% endfor -%} -auto-approve
         terraform output > $EUT_ROOT_DIR/{{ provider }}/site.tfvars
         {% endfor %}
  timeout: 1h 30m
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% for provider, values in providers -%}
{% for rte in values.rtes %}
# rte - {{ provider }} - {{ rte.name | replace(from="_", to="-")}} - destroy
rte-{{ provider }}-{{ rte.name | replace(from="_", to="-")}}-destroy:
  <<: *base
  stage: rte-destroy
  rules:
    - !reference [ .destroy_rules, rules ]
  script:
      - |
        #!/usr/bin/env bash
        cd $RTE_{{ provider | upper }}_{{ rte.name | upper }}_ROOT_DIR
        terraform init --backend-config="key=features/$FEATURE/$ENVIRONMENT/regression/environment/{{ provider }}"
        terraform destroy -var-file=$RTE_ROOT_TF_VAR_FILE -auto-approve
  timeout: 30m
  retry:
    max: 1
    when:
      - script_failure
      - stuck_or_timeout_failure
      - runner_system_failure
{% endfor -%}
{% endfor %}