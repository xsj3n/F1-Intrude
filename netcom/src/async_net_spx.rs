use core::panic;
use std::{sync::Arc, cell::RefCell};

use futures::future::join_all;
use rustls::{RootCertStore, ClientConfig};
use tokio::{net::{TcpStream, tcp}, io::{AsyncWriteExt, AsyncReadExt}};
use tokio::task::JoinHandle;
use crate::{Callback, net_spx::{ResponseString, HTTPResult}, ffi::{HttpResponseDataC, RequestandPermutation}, log::dbg_log_progress, DOMAIN_BUF};


// ASYNC RE_WRITE===
struct WorkerLoad
{
    work_grp_num: u32,
    tasks_per: u32,
    remainder: u32
}

pub async fn start_taskmaster(domain: String, request_perumation_buffer: RequestandPermutation, cb: Callback)
{


    let wrk_load = derive_WorkerLoad(request_perumation_buffer.request.len(), 100).await;
    let mut tasks = calc_tasks_per_worker(request_perumation_buffer, &wrk_load).await;

    let mut log_s = format!("[*] ===Starting Taskmaster:\nWorkers: {}  Tasks-per-worker:  {}  Total Permuations: {} Host: {}", wrk_load.work_grp_num, wrk_load.tasks_per, tasks.len(), &domain);
    dbg_log_progress(&log_s);
    
    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(
        webpki_roots::TLS_SERVER_ROOTS
            .iter()
            .cloned()
    );

    let mut j_v: Vec<JoinHandle<()>> = Vec::new();
    let len = tasks.len();

    for x in 0..len
    {
        let root_clone = root_store.clone();
        let domain_str_clone    = domain.clone();
        let task_ref = tasks.remove(0);

        log_s = format!("[*] ===Spawning worker {} on target domain {}...", x, &domain);
        dbg_log_progress(&log_s);
        
        let j = tokio::spawn( async move { start_worker(domain_str_clone, task_ref, root_clone,  cb.clone()).await; });
        j_v.push(j);
    }

    join_all(j_v).await;



}

thread_local!{ static ROW_LEVEL: Arc<RefCell<u16>> = Arc::new(RefCell::new(0)); }
async fn start_worker(d_s: String, request_perumation_buffer: RequestandPermutation, root_store: RootCertStore, cb: Callback) -> ()
{

    


    let tcp_stream = match TcpStream::connect(d_s.clone() + ":443").await
    {
        Ok(t) => t,
        Err(_) => 
        {

            let dbg_s = "[!] Worker unable to connect to ".to_string() + &d_s + ":443";
            dbg_log_progress(&dbg_s);
            return;
        }
    };

    println!("===Starting Worker...");
    let client_config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let conn = tokio_rustls::TlsConnector::from(Arc::new(client_config));


    let mut t = match conn.connect(d_s.clone().try_into().unwrap(), tcp_stream).await
    {
        Ok(t) => t,
        Err(_) => 
        {
            let dbg_s = "[!] Worker unable to connect to ".to_string() + &d_s;
            dbg_log_progress(&dbg_s);
            return;
        }
    };

   let mut i = 0;
    for rs in &request_perumation_buffer.request
    {
        t.write_all(rs.as_bytes()).await.unwrap();
        t.flush().await.unwrap();

        let mut b: Vec<u8> = Vec::new();
        let mut rd_buf = [0u8; 1024];
        
        loop 
        {
            let _bytes_read = t.read(&mut rd_buf[..]).await.unwrap();
            b.extend_from_slice(&rd_buf);
            
            let s = String::from_utf8_lossy(&b).to_string();
 
            match find_if_body(&s).await
            {
                ReadStatus::READ_AGAIN =>
                {
                    continue;
                },
                ReadStatus::DONE =>
                {
                    break;
                }
            };         
        }   

        let log_s = format!("[*] ===Worker received a response:\n{}", String::from_utf8_lossy(&b));
        dbg_log_progress(&log_s);
        
        i += 1;
        let final_response =  ResponseString(
            String::from_utf8_lossy(&b).to_string()
            ).parse_response();

        let _ = match cb 
        {
            Some(f) => 
            {
                match final_response
                {
                    Ok(hrdc)     => f(
                        hrdc, 
                        request_perumation_buffer.permutation[i].clone(), 
                        access_and_increment_rowlevel()),

                    Err(HTTPResult::TLS_READ_ERROR) => f(
                        HttpResponseDataC::new((None, None), 0, "[!] TLS Error".to_string()),
                        request_perumation_buffer.permutation[i].clone(),
                        access_and_increment_rowlevel()
                    ),

                    Err(_)                          => f(
                        HttpResponseDataC::new((None, None), 0, String::new()),
                        request_perumation_buffer.permutation[i].clone(),
                        access_and_increment_rowlevel())  
                }
            },
            None => panic!("===========NO CALLBACK PASSED=============")
        };

        return;

    }

}

fn access_and_increment_rowlevel() -> u16
{
    let row = ROW_LEVEL.with(|i: &Arc<RefCell<u16>>|
        {
            *i.borrow_mut() += 1;
            let i_in = *i.borrow_mut();
            i_in.clone() - 1
        });

    return row;
}


async fn derive_WorkerLoad(num: usize, requests_per_thread: usize) -> WorkerLoad
{

    let wrk: WorkerLoad = match num > 100
    {
        true =>
        {
            let worker_group_number = num / requests_per_thread;
            if num % 100 == 0
            {
                let (worker_group_number, tasks_per_worker): (usize, usize) = (worker_group_number, num / worker_group_number);
                WorkerLoad
                {
                    work_grp_num: worker_group_number as u32,
                    tasks_per: tasks_per_worker as u32,
                    remainder: 0
                }
                
            } else
            {
                let (worker_group_number, tasks_per_worker): (usize, usize) = (worker_group_number, num / worker_group_number);
                let remaining_tasks_to_be_distributed = num - (worker_group_number * tasks_per_worker);
                let  w = WorkerLoad
                {
                    work_grp_num: worker_group_number as u32,
                    tasks_per: tasks_per_worker as u32,
                    remainder: remaining_tasks_to_be_distributed as u32
                };
                w

            }
            
        }
        false =>
        {   
            WorkerLoad
            {
                work_grp_num: 1,
                tasks_per: num as u32,
                remainder: 0
            }
        }
    };

    return wrk;
    
}


async fn calc_tasks_per_worker(bulk: RequestandPermutation, wrk_load: &WorkerLoad) -> Vec<RequestandPermutation>
{
    let mut i = 1;
    let mut bot_cap = 0;
    let mut tasks: Vec<RequestandPermutation> = Vec::new();
    loop
    {
        let mut cap: usize = (wrk_load.tasks_per * i) as usize;
         
        let r_slice = bulk.request[bot_cap..cap].to_vec();
        let p_slice = bulk.permutation[bot_cap..cap].to_vec();

        tasks.push(RequestandPermutation { request: r_slice, permutation: p_slice });

        bot_cap = cap;
        cap += wrk_load.tasks_per as usize;
        
        i += 1;
        if i == wrk_load.work_grp_num + 1
        {
           return tasks;
        }
        
    }

}

enum ReadStatus
{
    DONE,
    READ_AGAIN
}
async fn find_if_body(s: &str) -> ReadStatus
{
    if !s.contains("Content-Length:")
    {
        return ReadStatus::DONE;

    } else if s.contains("\r\n\r\n")
    {
        let index = s.find("\r\n\r\n").unwrap();
        let body = &s[index + 4..];

        if body.len() <= 2
        {
            return ReadStatus::READ_AGAIN;

        } else
        {
            return ReadStatus::DONE;  
        }
        
    } else if s.contains("\n\n")
    {
        let index = s.find("\n\n").unwrap();
        let body = &s[index + 2..];
        
        if body.len() <= 2
        {
            return ReadStatus::READ_AGAIN; 

        } else
        {
            // stop reading
            return ReadStatus::DONE; 
        }
    } else
    {
        // Log un-conclusive request 
        return ReadStatus::DONE;
    }
}