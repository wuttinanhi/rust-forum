
# Rust Forum

A web forum application built with Rust, leveraging Actix-web framework and PostgreSQL.

## Features

- User authentication
- Forum posts and comments
- Session management
- Rate limiting
- CORS

## Getting Started

### Development Setup

1. Clone the repository
```bash
git clone https://github.com/yourusername/rust-forum.git
cd rust-forum
```

2. Copy environment file
```bash
cp .env.example .env
```

3. Run development setup script
```bash
./devsetup.sh
```

4. Start PostgreSQL and Redis
```bash
docker compose -f "docker-compose.dev.yml" up -d
```

5. Run database migrations
```bash
diesel migration run
```

6. Start development with watch
```bash
./watch.sh
```

### Production Deployment

1. Build and start all services
```bash
docker compose up -d --build
```

2. Run database migrations
```bash
diesel migration run
```

The application will be available at `http://localhost:3000`


## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
