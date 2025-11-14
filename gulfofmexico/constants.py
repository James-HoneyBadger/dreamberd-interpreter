"""Constants for the Gulf of Mexico interpreter."""

# Confidence and lifetime values
MAX_CONFIDENCE = 100000000000
DEFAULT_CONFIDENCE = 0
INFINITE_LIFETIME = 100000000000

# File storage paths
DB_RUNTIME_PATH = ".gulfofmexico_runtime"
DB_IMMUTABLE_CONSTANTS_PATH = ".immutable_constants"
DB_IMMUTABLE_CONSTANTS_VALUES_PATH = ".immutable_constants_values"
DB_VAR_TO_VALUE_SEP = ";;;"

# GitHub integration
GITHUB_GLOBAL_VARS_REPO = "GulfOfMexico/GulfOfMexico-Public-Variables"
GITHUB_GLOBAL_VARS_LABEL = "global variable"

# Precision settings
FLOAT_COMPARISON_EPSILON = 1e-10

# Cache settings
EXPRESSION_CACHE_SIZE = 1000
NAMESPACE_CACHE_SIZE = 500
