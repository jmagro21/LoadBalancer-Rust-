//! # Load Balancer en Rust avec Tokio
//!
//! Ce programme implémente un load balancer simple utilisant Tokio pour la gestion asynchrone.
//!
//! Changer les IPs cibles dans la fonction main.
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncWriteExt};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::error::Error;
use log::{info, error};

/// Représente un serveur backend.
#[derive(Clone)]
struct Backend {
    /// Adresse du serveur backend.
    address: String,
    /// Compteur du nombre de connexions actives vers ce backend.
    active_connections: Arc<Mutex<usize>>,
}

/// Vérifie la disponibilité d'un backend en tentant une connexion TCP.
///
/// # Arguments
///
/// * `backend_address` - L'adresse du serveur backend à vérifier.
async fn check_backend_available(backend_address: &String) -> bool {
    TcpStream::connect(backend_address).await.is_ok()
}

/// Sélectionne un backend disponible ayant le moins de connexions actives.
///
/// # Arguments
///
/// * `backends` - La liste des backends disponibles.
async fn select_backend(backends: &VecDeque<Backend>) -> Option<Backend> {
    let mut min_conn_backend = None;
    let mut min_conns = usize::MAX;

    for backend in backends {
        if check_backend_available(&backend.address).await {
            let conns = *backend.active_connections.lock().unwrap();
            if conns < min_conns {
                min_conn_backend = Some(backend.clone());
                min_conns = conns;
            }
        }
    }

    min_conn_backend
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let listener = TcpListener::bind("0.0.0.0:80").await?;
    let mut backends = VecDeque::new();

    // Initialisation des backends
    backends.push_back(Backend {
        address: "192.168.1.200:80".to_string(),
        active_connections: Arc::new(Mutex::new(0)),
    });
    backends.push_back(Backend {
        address: "192.168.1.27:80".to_string(),
        active_connections: Arc::new(Mutex::new(0)),
    });

    info!("Démarrage du load balancer sur le port 80");

    loop {
        let (socket, _) = listener.accept().await?;

        if let Some(backend) = select_backend(&backends).await {
            let active_connections = Arc::clone(&backend.active_connections);
            let backend_address = backend.address.clone();
            info!("Redirection de la connexion vers le backend {}", backend_address);

            tokio::spawn(async move {
                {
                    let mut active_conns = active_connections.lock().unwrap();
                    *active_conns += 1;
                }

                let handle_result = handle_connection(socket, backend_address).await;

                {
                    let mut active_conns = active_connections.lock().unwrap();
                    *active_conns -= 1;
                }

                if handle_result.is_err() {
                    error!("Erreur")
                }
            });
        } else {
            error!("Aucun backend disponible pour gérer la connexion");
        }
    }
}

/// Gère une connexion entrante en la transférant à un backend.
///
/// # Arguments
///
/// * `socket` - Le socket de la connexion entrante.
/// * `backend` - L'adresse du backend vers lequel la connexion doit être redirigée.
async fn handle_connection(mut socket: TcpStream, backend: String) -> Result<(), Box<dyn Error + Send>> {
    match TcpStream::connect(&backend).await {
        Ok(mut backend_socket) => {
            let (mut ri, mut wi) = socket.split();
            let (mut ro, mut wo) = backend_socket.split();

            let client_to_server = io::copy(&mut ri, &mut wo);
            let server_to_client = io::copy(&mut ro, &mut wi);

            tokio::select! {
                result = client_to_server => {
                    result.map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
                    wo.shutdown().await.map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
                },
                result = server_to_client => {
                    result.map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
                    wi.shutdown().await.map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;
                }
            }
            Ok(())
        },
        Err(e) => {
            error!("Échec de la connexion au backend {}: {:?}", backend, e);
            Err(Box::new(e) as Box<dyn Error + Send>)
        }
    }
}
