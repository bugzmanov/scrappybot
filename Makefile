build: 
	docker run -e BIN=bootstrap   -v ${PWD}:/code     -v ${HOME}/.cargo/registry:/root/.cargo/registry     -v ${HOME}/.cargo/git:/root/.cargo/git   softprops/lambda-rust

deploy:
	echo "TODO"