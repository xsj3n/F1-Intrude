use core::panic;
use std::{sync::Arc, cell::RefCell};

use futures::future::join_all;
use rustls::{RootCertStore, ClientConfig};
use tokio::{net::{TcpStream, tcp}, io::{AsyncWriteExt, AsyncReadExt}, sync::Mutex};
use tokio::task::JoinHandle;
use crate::{Callback, net_spx::{ResponseString, HTTPResult}, ffi::{HttpResponseDataC, RequestandPermutation}, log::dbg_log_progress, DOMAIN_BUF};


// ASYNC RE_WRITE===
struct WorkerLoad
{
    work_grp_num: u32,
    tasks_per: u32,
    remainder: u32
}

enum HttpStatus 
{
    FullyConstructedHeaderOnly,
    FullyConstructed,
    NotDone
}

async fn start_worker(d_s: String, request_perumation_buffer: RequestandPermutation, root_store: RootCertStore, acc_v: Arc<Mutex<Vec<String>>>) -> ()
{
    
    // this just lets jus boot our connection back up if we get our connection closed on us
    // this can happen if the server returns a 404 and insta-closes 
    let mut straggler_kq_v: Vec<JoinHandle<()>> = Vec::new();
    let mut resume = 0; // -1 as it will be used for indexing
    'worker_start: loop {
        let tcp_stream = match TcpStream::connect(d_s.clone() + ":443").await
        {
            Ok(t) => t,
            Err(_) => 
            {
    
                return;
            }
        };
        tcp_stream.set_nodelay(true).unwrap();
    
        println!("===Starting Worker...");
        let client_config = ClientConfig::builder()
            .with_root_certificates(root_store.clone())
            .with_no_client_auth();
    
        let conn = tokio_rustls::TlsConnector::from(Arc::new(client_config));
    
    
        let mut t = match conn.connect(d_s.clone().try_into().unwrap(), tcp_stream).await
        {
            Ok(t) => t,
            Err(_) => 
            {
                let dbg_s = "[!] Worker unable to connect to ".to_string() + &d_s;
                println!("{}", &dbg_s);
                return;
    
                
            }
        };
        
        
        'out: for rs in &request_perumation_buffer.request[resume..]
        {
            if resume == request_perumation_buffer.request.len()
            {
                return;
            }
            t.write_all(rs.as_bytes()).await.unwrap();
            //println!("Written:\n{}", &rs);
            t.flush().await.unwrap();
            
    
            let mut b: Vec<u8> = Vec::new();
            let mut rd_buf = [0u8; 4096];
            
            loop 
            {
                

                // failing to avoid this read when there is nothing left, is e v e r y thing
                let _bytes_read = t.read(&mut rd_buf[..]).await.unwrap();
                if _bytes_read == 0 
                { 
                    println!("Bytes read: {}", _bytes_read);
                    straggler_kq_v.push(kq_straggler(d_s.clone(), &rs, root_store.clone(), acc_v.clone()));
                    resume += 1;
                    continue 'worker_start;
                } // TODO: we would log a failure
                b.extend_from_slice(&rd_buf[.._bytes_read]);
           
                
               match chk_if_http_is_done(&b).await
               {
                    HttpStatus::FullyConstructed => 
                    {
                        let fin = String::from_utf8_lossy(&b)
                            .to_string();
                        //println!("Valid:\n{}", &fin);
                        resume += 1;
                        acc_v.lock().await.push(fin);
                        continue 'out;
                    }
    
                    HttpStatus::FullyConstructedHeaderOnly =>
                    {
                        let fin = String::from_utf8_lossy(&b)
                            .to_string();
                        //println!("Valid:\n{}", &fin);
                        acc_v.lock().await.push(fin);
                        resume += 1;
                        continue 'out;
                    }
                    
                    HttpStatus::NotDone => continue
               }
     
               
            }   
    
            
            
    
        }
        break;
    }
    join_all(straggler_kq_v).await; 
    return;
}

fn access_and_increment_rowlevel() -> ()//u16
{
    /* 
    let row = ROW_LEVEL.with(|i: &Arc<RefCell<u16>>|
        {
            *i.borrow_mut() += 1;
            let i_in = *i.borrow_mut();
            i_in.clone() - 1
        });

    return row;
    */
}


#[inline(always)]
// perhaps CL can represenrt the bytes left to read
async fn chk_if_http_is_done(accum: &[u8]) -> HttpStatus
{


    let response = String::from_utf8_lossy(&accum).to_string();
    let target_len  = chk_content_length(&accum).await;
    let current_len = determine_body_sz_in_accum(&accum).await;

    //println!("{} out of {} body bytes read!", current_len, target_len);

    if response.len() != 0 
    {
        //assert!(response.contains("HTTP/1.1"));
    }


    if response.contains("\r\n\r\n") && !response.contains("Content-Length") && !response.contains("content-length")
    {
        //println!("Valid-HO:\n{}", response);
        return HttpStatus::FullyConstructedHeaderOnly; // No body, message end 
        
    }

    if response.contains("\r\n\r\n") && target_len <= current_len
    {
        //println!("Valid:\n{}", response);
        return HttpStatus::FullyConstructed;
    }

    return HttpStatus::NotDone; // Incomplete response, read more;
}

#[inline(always)]
async fn chk_content_length(accum: &[u8]) -> isize
{
    let response = String::from_utf8_lossy(&accum).to_string();
    let lines = response.split("\r\n");
    for l in lines
    {
        if response.contains("HTTP/1.1") &&
        (l.contains("Content-Length") || l.contains("content-length")) && response.contains("\r\n\r\n") 
        {
            let body_len = if l.contains("Content-Length") 
            {
                l.replace("Content-Length: ", "").trim()
                    .parse::<isize>().unwrap()
            } else 
            {
                l.replace("content-length: ", "").trim()
                    .parse::<isize>().unwrap()
            };     
            return body_len as isize; // there is a body, and it is next
        }
    }

    if response.contains("HTTP/1.1") && response.contains("\r\n\r\n")
    {
        return 0; // Response done, only the header
    }

    return -1; // return -1 when not even the full http header has been received 
}


#[inline(always)]
async fn determine_body_sz_in_accum(accum: &[u8]) -> isize
{
    let response = String::from_utf8_lossy(&accum).to_string();
    let sub_strs = response.split("\r\n\r\n");

    for half in sub_strs
    {
        
        if !half.contains("HTTP/1.1") && !half.is_empty()
        {
            return half.len().try_into().unwrap();
        }
        
    }

    return 0; //failure or headers only
}


fn kq_straggler(d_s: String,rs: &str, root_store: RootCertStore, acc_v: Arc<Mutex<Vec<String>>>) -> JoinHandle<()>
{
    let r = RequestandPermutation
    {
        request: vec![rs.to_string(); 1],
        permutation: vec!["perm".to_string(); 1]
    };

    println!("Spawning KQ Task due to connection closed>>>>");
    return tokio::spawn(async move 
        {
            start_worker(d_s, r, root_store, acc_v).await;
        });
}