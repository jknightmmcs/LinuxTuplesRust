
use std::net::*;
use std::io::prelude::*;
use std::mem::*;
use std::vec::Vec;


/// Enum, which is used to contain LinuxTuples element
#[derive(Clone, PartialEq)]
pub enum E {
    I(i32),
    D(f64),
    S(String),
    T(Vec<E>),
    None,
}

impl E {
	/// Prints an element to the standard output
	/// # Examples
	///
	/// ```
	/// # e.print()
	/// ```
	pub fn print(&self) {
		match self {
			&E::I(ref i) => print!("Int: {}, ", i),
			&E::D(ref d) => print!("Double: {}, ", d),
			&E::S(ref s) => print!("String: {}, ", s),
			&E::T(ref v) => {
				print!("Tuple: [");
				for e in v {
					e.print();
				}
				print!("], ");
				
			}
			&E::None => println!("Wildcard"),
		}
	}
}

/// Used to connect to LinuxTuples server
///
/// # Examples
///
/// ```
/// let conn = LinuxTuplesConnection { addr };
/// ``` 
#[derive(Clone)]
pub struct LinuxTuplesConnection {
	pub connection: SocketAddr,
}


const PUT: i32 = 0;
const GET: i32 = 1;
const READ: i32 = 2;
const GET_NB: i32 = 3;
const READ_NB: i32 = 4;
const DUMP: i32 = 5;
const COUNT: i32 = 6;
const REPLACE: i32 = 8;
const LOG: i32 = 7;

const ASCII_S: i32 = 115;
const ASCII_I: i32 = 105;
const ASCII_D: i32 = 100;
const ASCII_Q: i32 = 63;
const ASCII_T: i32 = 116;

fn ti(i: i32) -> [u8; 4] {
	let buff: [u8; 4];
	unsafe {
		buff = transmute(i);
	}
	return buff;
} 

fn ctoi(arr: &mut [u8; 24], mut val: i32, offset: isize) {
	unsafe {
		::std::ptr::copy_nonoverlapping(&mut val as *mut _ as *mut u8, arr.as_ptr().offset(offset) as *mut u8, ::std::mem::size_of::<i32>());
	}
}

fn ciot(arr:&[u8; 24], offset:isize) -> i32 {
	let mut val: i32 = 0;
	
	unsafe {
		::std::ptr::copy_nonoverlapping(arr.as_ptr().offset(offset) as *mut u8, &mut val as *mut _ as *mut u8, ::std::mem::size_of::<i32>());
	}
	
	return val;
}

fn cdot(arr:&[u8; 24], offset:isize) -> f64 {
	let mut val: f64 = 0.;
	unsafe {
		::std::ptr::copy_nonoverlapping(arr.as_ptr().offset(offset) as *mut u8, &mut val as *mut _ as *mut u8, ::std::mem::size_of::<f64>());
	}
	return val;
}

fn ctod(arr: &mut [u8; 24], mut val: f64, offset: isize) {
	unsafe {
		::std::ptr::copy_nonoverlapping(&mut val as *mut _ as *mut u8, arr.as_ptr().offset(offset) as *mut u8, ::std::mem::size_of::<f64>());
	}
}  

fn send_tuple(tuple: &Vec<E>, stream: &mut TcpStream) {
	
    stream.write_all(& (ti(tuple.len() as i32)));
    
    let mut string_length:i32 = 0;
    
    for elem in tuple {
    	match elem {
    		&E::S(ref s) => string_length += s.len() as i32,
    		_ => {}
    	}
    }
    
    stream.write_all(&ti(string_length));
    
    string_length = 0;
    
    let mut buff: [u8; 24] = [0; 24];
    
    for elem in tuple {
    	match elem {
    		&E::I(ref i) => {
				//tag
   	 			ctoi(&mut buff, ASCII_I, 0);
   	 			//union
   	 			ctoi(&mut buff, *i, 8);
   	 			
   	 			stream.write_all(&buff); 
    		},
    		&E::D(ref d) => {
    			//tag
   	 			ctoi(&mut buff, ASCII_D, 0);
   	 			//union
   	 			ctod(&mut buff, *d, 8);
   	 			stream.write_all(&buff);
   	 			
    		},
    		&E::S(ref s) => {
    			//tag
   	 			ctoi(&mut buff, ASCII_S, 0);
   	 			
   	 			//union
   	 			ctoi(&mut buff, string_length, 8);
   	 			ctoi(&mut buff, s.len() as i32, 16);
   	 			
   	 			//tuple pointer
   	 			stream.write_all(&buff);
   	 			
   	 			string_length += s.len() as i32;
    		},
    		&E::T(ref v) => {
    			ctoi(&mut buff, ASCII_T, 0);
    			
    			stream.write_all(&mut buff);
    			
    			send_tuple(v, stream);
    		}
    		&E::None => {
    			ctoi(&mut buff, ASCII_Q, 0);
    			
    			stream.write_all(&buff);
    		}
    	}
    }
    
    for elem in tuple {
    	match elem {
    		&E::S(ref s) => {
    			stream.write_all(s.as_bytes());
    		}
    		_ => {}
    	}
    }
}

fn recv_tuple(stream: &mut TcpStream) -> Vec<E> {
	
	struct str_desc {
		used: bool,
		offset: i32,
		len: i32,
	}
	
	let mut tuple: Vec<E> = Vec::<E>::new();
	let mut str_descs: Vec<str_desc> = Vec::<str_desc>::new();
	
	let mut buff_4:[u8; 4] = [0; 4];
	let mut buff_8:[u8; 8] = [0; 8];
	let mut buff:[u8; 24] = [0; 24];
	
	stream.read_exact(&mut buff_4);
	let num_elements:i32 = unsafe { transmute(buff_4) };
	
	//non-blocking reads/writes
	if num_elements == -1 {
		return tuple;
	}
	
	stream.read_exact(&mut buff_4);
	let string_length:i32 = unsafe { transmute(buff_4) };
	
	for i in 0..num_elements {
		
		stream.read_exact(&mut buff);
		
		let tag: i32 = ciot(&buff, 0);
		
		match tag {
			ASCII_I => {
				tuple.push(E::I(ciot(&buff, 8)));
				str_descs.push(str_desc { used: false, offset: 0, len: 0} );
			},
			ASCII_D => {
				tuple.push(E::D(cdot(&buff, 8)));
				str_descs.push(str_desc { used: false, offset: 0, len: 0} );
			},
			ASCII_S => {
				str_descs.push(str_desc { 
						used: true,
						offset: ciot(&buff, 8),
						len: ciot(&buff, 16),
					});
				tuple.push(E::S("".to_string()));
			},
			ASCII_T => {
				str_descs.push(str_desc { used: false, offset: 0, len: 0 });
				tuple.push(E::T(recv_tuple(stream)));
			}
			ASCII_Q => {
				str_descs.push(str_desc { used: false, offset: 0, len: 0} );
			},
			_ => {
				
			}
		}
	}
	
	if string_length != 0 {
		let mut string_space:Vec<u8> = vec![0; string_length as usize];
		stream.read_exact(&mut string_space);
		
		let mut str_slice = string_space.as_slice();
		for i in 0..num_elements {
			if str_descs[i as usize].used == true {
				tuple[i as usize] = E::S(String::from_utf8(
					string_space[
						(str_descs[i as usize].offset as usize)..
						(str_descs[i as usize].offset + str_descs[i as usize].len) as usize].to_vec()).unwrap());
			}
		}
		
	}
	
	return tuple;
}

impl LinuxTuplesConnection {
	/// Puts tuple to the tuple space
	///
	/// # Examples
	///
	/// ```
	/// conn.put_tuple(&vec![E::I(123), E::S("123".to_string())]);
	/// ```
	///
	/// # Errors
	///
	/// Returns Err if disconnected, or incorrect data arrived
	pub fn put_tuple(&self, tuple: &Vec<E>) -> std::io::Result<bool> {
	    let mut stream_err = TcpStream::connect(&self.connection);
	    match stream_err {
	    	Ok(mut stream) => {
	    		stream.write_all(&ti(PUT));
				send_tuple(tuple, &mut stream);
				stream.shutdown(Shutdown::Write);
				return Ok(true);
	    	}
	    	Err(why) => {
	    		return Err(why);
	    	}
	    }
	}
	
	/// Gets tuple from the tuple space
	///
	/// # Examples
	///
	/// ```
	/// let tuple = conn.get_tuple(&vec![E::I(123), E::None]).unwrap();
	/// ```
	/// 
	/// # Errors
	///
	/// Returns Err if disconnected, or incorrect data arrived
	pub fn get_tuple(&self, tuple: &Vec<E>) -> std::io::Result<Vec<E>>
	{
		let mut stream_err = TcpStream::connect(&self.connection);
		match stream_err {
			Ok(mut stream) => {
				stream.write_all(&ti(GET));
				
				send_tuple(tuple, &mut stream);
				
				stream.shutdown(Shutdown::Write);
				
				return Ok(recv_tuple(&mut stream));
			}
			Err(why) => {
				return Err(why);
			}
		}
	}
	
	/// Reads tuple from the tuple space
	///
	/// # Examples
	///
	/// ```
	/// let tuple = conn.read_tuple(&vec![E::I(123), E::None]).unwrap();
	/// ```
	/// # Errors
	///
	/// Returns Err if disconnected, or incorrect data arrived
	pub fn read_tuple(&self, tuple: &Vec<E>) -> std::io::Result<Vec<E>>
	{
		let mut stream_err = TcpStream::connect(&self.connection);
		match stream_err {
			Ok(mut stream) => {
				stream.write_all(&ti(READ));
				
				send_tuple(tuple, &mut stream);
				
				stream.shutdown(Shutdown::Write);
				
				return Ok(recv_tuple(&mut stream));
			}
			Err(why) => {
				return Err(why);
			}
		}
	}
	
	/// Gets tuple from the tuple space; non-blocking
	///
	/// If nothing found, returns empty vector.
	/// # Examples
	///
	/// ```
	/// let tuple = conn.get_nb_tuple(&vec![E::I(123), E::S("123".to_string())]).unwrap();
	/// ```
	///
	/// # Errors
	///
	/// Returns Err if disconnected, or incorrect data arrived
	pub fn get_nb_tuple(&self, tuple: &Vec<E>) -> std::io::Result<Vec<E>>
	{
		let mut stream_err = TcpStream::connect(&self.connection);
		match stream_err {
			Ok(mut stream) => {
				stream.write_all(&ti(GET_NB));
				
				send_tuple(tuple, &mut stream);
				
				stream.shutdown(Shutdown::Write);
				
				return Ok(recv_tuple(&mut stream));
			}
			Err(why) => {
				return Err(why);
			}
		}
	}
	
	/// Reads tuple from the tuple space; non-blocking
	///
	/// If nothing found, returns empty vector.
	///
	/// # Examples
	///
	/// ```
	/// let tuple = conn.read_nb_tuple(&vec![E::I(123), E::S("123".to_string())]).unwrap();
	/// ```
	///
	/// # Errors
	///
	/// Returns Err if disconnected, or incorrect data arrived
	pub fn read_nb_tuple(&self, tuple: &Vec<E>) -> std::io::Result<Vec<E>>
	{
		let mut stream_err = TcpStream::connect(&self.connection);
		match stream_err {
			Ok(mut stream) => {
				stream.write_all(&ti(READ_NB));
				
				send_tuple(tuple, &mut stream);
				
				stream.shutdown(Shutdown::Write);
				
				return Ok(recv_tuple(&mut stream));
			}
			Err(why) => {
				return Err(why);
			}
		}
	}
	
	/// Reads all tuples from the tuple space that satisfy templates
	///
	/// If nothing found, returns empty vector. If template list is empty, returns all tuples in the space.
	///
	/// # Examples
	///
	/// ```
	/// let tuple_list = conn.read_all_tuples(&vec![vec![E::I(123), E::None], vec![E::S("123".to_string()), E::None]]).unwrap();
	/// ```
	///
	/// # Errors
	///
	/// Returns Err if disconnected, or incorrect data arrived
	pub fn read_all_tuples(&self, tuples: &Vec<Vec<E>>) -> std::io::Result<Vec<Vec<E>>>
	{
		let mut stream_err = TcpStream::connect(&self.connection);
		match stream_err {
			Ok(mut stream) => {
				stream.write_all(&ti(DUMP));
				let count: i32 = tuples.len() as i32;
				stream.write_all(&ti(count));
				
				for tuple in tuples {
					send_tuple(tuple, &mut stream);
				}
				
				let mut buff_4:[u8; 4] = [0; 4];
				
				stream.read_exact(&mut buff_4);
				
				let recv_count = unsafe { transmute(buff_4) };
				
				let mut result:Vec<Vec<E>> = Vec::<Vec<E>>::new();
				
				for i in 0..recv_count {
					result.push(recv_tuple(&mut stream));
				}
				return Ok(result);
			}
			Err(why) => {
				return Err(why);
			}
		}
	}
	
	/// Calculates a number tuples from the tuple space that satisfy templates
	///
	/// If nothing found, 0. If template list is empty, returns number of all tuples in the space.
	///
	/// # Examples
	///
	/// ```
	/// let count = conn.number_of_tuples(&vec![vec![E::I(123), E::None], vec![E::S("123".to_string()), E::None]]).unwrap();
	/// ```
	///
	/// # Errors
	///
	/// Returns Err if disconnected, or incorrect data arrived
	pub fn number_of_tuples(&self, tuples: &Vec<Vec<E>>) -> std::io::Result<i32>
	{
		let mut stream_err = TcpStream::connect(&self.connection);
		match stream_err {
			Ok(mut stream) => {
				stream.write_all(&ti(COUNT));
				let count: i32 = tuples.len() as i32;
				stream.write_all(&ti(count));
				
				for tuple in tuples {
					send_tuple(tuple, &mut stream);
				}
				
				let mut buff_4:[u8; 4] = [0; 4];
				
				stream.read_exact(&mut buff_4);
				
				let recv_count = unsafe { transmute(buff_4) };
				
				return Ok(recv_count);
			}
			Err(why) => {
				return Err(why);
			}
		}
	}
	
	/// Replaces one tuple with another in the space
	///
	///
	/// # Examples
	///
	/// ```
	/// conn.replace_tuple(&vec![E::I(123)], &vec![E::S("123".to_string())]).unwrap();
	/// ```
	///
	/// # Errors
	///
	/// Returns Err if source tuple wasn't found or data was corrupted/socket disconnected.
	pub fn replace_tuple(&self, tuple: &Vec<E>, replacement: &Vec<E>) -> std::io::Result<bool> {
			let mut stream_err = TcpStream::connect(&self.connection);
			match stream_err {
				Ok(mut stream) => {
					stream.write_all(&ti(REPLACE));
					send_tuple(tuple, &mut stream);
					send_tuple(replacement, &mut stream);
					let mut buff_4:[u8; 4] = [0; 4];
					stream.read_exact(&mut buff_4);
					
					let op:i32 = unsafe { transmute(buff_4) };
					if op != REPLACE {
						return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Replace failed"));
					} else {
						return Ok(true);
					}
				}
				Err(why) => {
					return Err(why);
					
				}
			}
	}
	/// Returns a list of operations done to a tuple.
	///
	/// # Examples
	///
	/// ```
	/// let log = conn.server_log();
	/// ```
	/// # Errors
	///
	/// Returns Err if disconnected, or incorrect data arrived
	pub fn server_log(&self) -> std::io::Result<String> {
		let mut stream_err = TcpStream::connect(&self.connection);
		match stream_err {
			Ok(mut stream) => {
				stream.write_all(&ti(LOG));
				let mut log:String = "".to_string();
				stream.read_to_string(&mut log);
				return Ok(log);
			}
			Err(why) => {
				return Err(why);
			}
		}
	}
	
	
	///Prints a tuple to the standard output
	///
	/// # Examples
	/// ```
	/// LinuxTuplesConnection::print_tuple(&vec![E::I(123)]);
	/// ```
	pub fn print_tuple(tuple: &Vec<E>) {
				print!("Tuple: [");
				for e in tuple {
					e.print();
				}
				print!("], ");
	}
}
