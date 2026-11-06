BEGIN;

-- Clean existing mock users and cascading posts and comments
DELETE FROM users WHERE email IN ('alice@example.com', 'bob@example.com', 'charlie@example.com');

-- Insert 3 users (password: 'password123')
INSERT INTO users (name, email, password, role)
VALUES
  ('Alice Johnson', 'alice@example.com', '$2b$12$qHzds3tX1JRdl7KyhtkoZ.Mph5IZnyADtJRJk4Uy7lZJXDdqAxduu', 'user'),
  ('Bob Smith', 'bob@example.com', '$2b$12$qHzds3tX1JRdl7KyhtkoZ.Mph5IZnyADtJRJk4Uy7lZJXDdqAxduu', 'user'),
  ('Charlie Brown', 'charlie@example.com', '$2b$12$qHzds3tX1JRdl7KyhtkoZ.Mph5IZnyADtJRJk4Uy7lZJXDdqAxduu', 'user');

-- Insert 3 posts (one for each user)
INSERT INTO posts (title, body, published, user_id)
VALUES
  (
    'Getting Started with Rust',
    'Rust is a systems programming language that provides memory safety without garbage collection.',
    TRUE,
    (SELECT id FROM users WHERE email = 'alice@example.com')
  ),
  (
    'Web Development with Actix Web',
    'Actix Web is a powerful, pragmatic, and extremely fast web framework for Rust.',
    TRUE,
    (SELECT id FROM users WHERE email = 'bob@example.com')
  ),
  (
    'Database Persistence with Diesel',
    'Diesel provides safe and extensible database interactions with compile-time checked queries.',
    TRUE,
    (SELECT id FROM users WHERE email = 'charlie@example.com')
  );

-- Insert 5 comments for Post 1
INSERT INTO comments (content, post_id, user_id)
VALUES
  (
    'Welcome to the Rust community! The borrow checker takes time, but it is worth it.',
    (SELECT id FROM posts WHERE title = 'Getting Started with Rust'),
    (SELECT id FROM users WHERE email = 'bob@example.com')
  ),
  (
    'I recommend reading The Rust Programming Language book to get up to speed.',
    (SELECT id FROM posts WHERE title = 'Getting Started with Rust'),
    (SELECT id FROM users WHERE email = 'charlie@example.com')
  ),
  (
    'Cargo makes package management so enjoyable compared to other ecosystems.',
    (SELECT id FROM posts WHERE title = 'Getting Started with Rust'),
    (SELECT id FROM users WHERE email = 'bob@example.com')
  ),
  (
    'Do not forget to run clippy on your projects for idiomatic code suggestions.',
    (SELECT id FROM posts WHERE title = 'Getting Started with Rust'),
    (SELECT id FROM users WHERE email = 'charlie@example.com')
  ),
  (
    'Thanks everyone for the helpful advice and warm welcome!',
    (SELECT id FROM posts WHERE title = 'Getting Started with Rust'),
    (SELECT id FROM users WHERE email = 'alice@example.com')
  );

-- Insert 5 comments for Post 2
INSERT INTO comments (content, post_id, user_id)
VALUES
  (
    'Actix Web has been rock solid in our production environment.',
    (SELECT id FROM posts WHERE title = 'Web Development with Actix Web'),
    (SELECT id FROM users WHERE email = 'alice@example.com')
  ),
  (
    'How does Actix Web compare to Axum in terms of developer ergonomics?',
    (SELECT id FROM posts WHERE title = 'Web Development with Actix Web'),
    (SELECT id FROM users WHERE email = 'charlie@example.com')
  ),
  (
    'Actix Web offers fantastic raw throughput and mature documentation.',
    (SELECT id FROM posts WHERE title = 'Web Development with Actix Web'),
    (SELECT id FROM users WHERE email = 'bob@example.com')
  ),
  (
    'The middleware ecosystem in Actix Web is also very straightforward to extend.',
    (SELECT id FROM posts WHERE title = 'Web Development with Actix Web'),
    (SELECT id FROM users WHERE email = 'alice@example.com')
  ),
  (
    'I will definitely give it a try on my next service.',
    (SELECT id FROM posts WHERE title = 'Web Development with Actix Web'),
    (SELECT id FROM users WHERE email = 'charlie@example.com')
  );

-- Insert 5 comments for Post 3
INSERT INTO comments (content, post_id, user_id)
VALUES
  (
    'Compile-time query validation has saved us from so many runtime bugs.',
    (SELECT id FROM posts WHERE title = 'Database Persistence with Diesel'),
    (SELECT id FROM users WHERE email = 'alice@example.com')
  ),
  (
    'Do you prefer writing raw SQL migrations or diesel migration generator?',
    (SELECT id FROM posts WHERE title = 'Database Persistence with Diesel'),
    (SELECT id FROM users WHERE email = 'bob@example.com')
  ),
  (
    'Diesel migrations make schema versioning and deployment straightforward.',
    (SELECT id FROM posts WHERE title = 'Database Persistence with Diesel'),
    (SELECT id FROM users WHERE email = 'charlie@example.com')
  ),
  (
    'The connection pooling with r2d2 integration works seamlessly.',
    (SELECT id FROM posts WHERE title = 'Database Persistence with Diesel'),
    (SELECT id FROM users WHERE email = 'alice@example.com')
  ),
  (
    'Great write-up! Looking forward to more Diesel content.',
    (SELECT id FROM posts WHERE title = 'Database Persistence with Diesel'),
    (SELECT id FROM users WHERE email = 'bob@example.com')
  );

COMMIT;
