build-all:
	set -e; \
	for src in ./sources/*; do \
		( \
			cd "$$src" && aidoku package \
		); \
	done ; \
	aidoku build sources/*/package.aix --name "caffing sources"
