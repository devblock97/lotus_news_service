# 1. Install sqlx-cli (used to run migrations). We exclude default features 
# and explicitly use the 'postgres' feature to keep the install clean and fast.
cargo install sqlx-cli --features postgres --no-default-features --root /usr/local;

# 2. Set the PATH so the shell can find the installed sqlx-cli binary
export PATH="$PATH:/usr/local/bin";

# 3. Run the migrations. This command automatically uses the DATABASE_URL 
# environment variable set in your Render settings.
sqlx migrate run;

# 4. Build the final release binary for your application
cargo build --release