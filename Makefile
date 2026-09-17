STRATUM ?= $(HOME)/.wine32/drive_c/Program Files/Stratum
HELPDECO ?= helpdeco

.PHONY: all corpus help lang texts icons check clean

all: lang texts check

## copy library, samples, templates and help out of an installed Stratum
corpus:
	tools/import_corpus.sh "$(STRATUM)"

## decompile SC3.HLP into docs/help and rebuild the merged function reference
help:
	tools/decompile_help.sh "$(HELPDECO)"

## compiler tables -> docs/lang/{builtins,operators,constants,tdl}.json
lang:
	python3 tools/tpl2json.py fixtures/template docs/lang fixtures/library
	@test -f docs/help/functions.json && \
	    python3 tools/merge_functions.py docs/lang/builtins.json docs/help/functions.json docs/lang || \
	    echo "docs/help/functions.json missing - run 'make help' to add descriptions"

## model text of every image -> docs/corpus
texts:
	python3 tools/extract_texts.py fixtures docs/corpus

## icon sheets -> PNG
icons:
	python3 tools/dbm2png.py --all fixtures/ICONS docs/icons

## parse every binary file in the corpus, then look for unknown identifiers
check:
	python3 tools/cls_dump.py --scan fixtures
	python3 tools/spj_dump.py --scan fixtures
	python3 tools/check_corpus.py docs/corpus docs/lang

clean:
	rm -rf docs/corpus docs/icons
