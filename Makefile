FRONTEND_DIR=frontend
STATIC_JS_DIR=static/js
STATIC_CSS_DIR=static/css

all: build

build: build-frontend build-backend

build-frontend:
	@echo "Building React frontend..."
	cd $(FRONTEND_DIR) && npm install && npm run build
	@echo "Copying React build files to Rust's static directories..."
	cp $(FRONTEND_DIR)/build/index.html $(STATIC_JS_DIR)/index.html
	cp $(FRONTEND_DIR)/build/static/js/main*.js $(STATIC_JS_DIR)/
	cp $(FRONTEND_DIR)/build/static/css/main*.css $(STATIC_CSS_DIR)/
	cp $(FRONTEND_DIR)/build/asset-manifest.json $(STATIC_JS_DIR)/asset-manifest.json

build-backend:
	@echo "Building Rust backend..."
	cargo build

run: build
	@echo "Running the Rust Server..."
	cargo run

clean:
	@echo "Cleaning React & Rust build artifcats..."
	rm -rf $(STATIC_JS_DIR)/main*.js
	rm -rf $(STATIC_CSS_DIR)/main*.css
	rm -rf $(STATIC_JS_DIR)/asset-manifest.json
	rm -rf $(FRONTEND_DIR)/node_modules/
	rm -rf $(FRONTEND_DIR)/build/
	cargo clean

watch:
	@echo "Starting watch for React frontend..."
	cd $(FRONTEND_DIR) && npm run start
