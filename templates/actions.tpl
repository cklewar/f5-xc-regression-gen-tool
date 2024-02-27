{
    "tests": {{ actions.tests | json_encode(pretty=true) | safe }},
    "verifications": {{ actions.verifications | json_encode(pretty=true) | safe }}
}