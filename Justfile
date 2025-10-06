# Utility tasks for meshbbs contributors

default:
	@just --list

utf8-check limit=200 *paths:
	python3 scripts/check_utf8_budget.py --limit {{limit}} {{paths}}

qa-utf8:
	python3 scripts/check_utf8_budget.py --limit 200 docs/qa tests/test-data-int
