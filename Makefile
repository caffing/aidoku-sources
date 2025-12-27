build-all:
	set -e; \
	for src in ./sources/*; do \
		( \
			aidoku package "$$src" \
		); \
	done ; \
	aidoku build sources/*/package.aix --name "caffing sources"
