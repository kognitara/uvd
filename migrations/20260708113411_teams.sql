-- Add migration script here
-- Table des Développeurs (Syntaxe MariaDB)
CREATE TABLE IF NOT EXISTS developers (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    gpg_key_id VARCHAR(255)
);

-- Table des Reviewers (Syntaxe MariaDB)
CREATE TABLE IF NOT EXISTS reviewers (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE
);
-- Table des Managers (Syntaxe MariaDB)
CREATE TABLE IF NOT EXISTS managers (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE
);
