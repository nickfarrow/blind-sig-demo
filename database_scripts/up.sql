CREATE TABLE nonces (
    secret_nonce VARCHAR(255) NOT NULL,
	public_nonce VARCHAR(255) NOT NULL
);

CREATE TABLE messages (
    msg VARCHAR(255) NOT NULL
);
