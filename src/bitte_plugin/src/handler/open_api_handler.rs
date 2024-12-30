use rocket::serde::json::{json, Json, Value};
use std::env;

use crate::handler::utils::extract_plugin_url;

/// Route handler for serving the OpenAPI specification
#[get("/ai-plugin.json")]
pub fn open_api_specification() -> Json<Value> {
    let account_id = env::var("ACCOUNT_ID").expect("ACCOUNT_ID not defined");
    let plugin_url = extract_plugin_url();

    println!("Bitte AI Plugin Account ID: {}", account_id);
    println!("Bitte AI Plugin URL: {}", plugin_url);

    // Create a JSON object representing the OpenAPI specification
    let routes = json!({
            "/api/tweet": {
              "get": {
                "tags": [
                  "x.com",
                  "tweet",
                  "tweet Id",
                  "Reward",
                  "post",
                  "clone",
                  "collect",
                  "Snapshot",
                  "processed",
                  "tweet-snapshot",
                  "snapshot",
                  "Craft",
                  "Reward",
                  "Derive",
                  "Duplicate",
                ],
                "summary": "Retrieve data for a specific X / Tweet post",
                "description": "This endpoint provides an image and description of a post/tweet to X (Twitter) along with the cost to mint a unique 1 of 1 NFT representing the Post.",
                "operationId": "tweet-snapshot",
                "parameters": [
                  {
                    "name": "tweet_id",
                    "in": "query",
                    "description": "The tweet ID of the post to be rewarded (Note: It is a 19-digit long numeric ID)",
                    "required": true,
                    "schema": {
                      "type": "string"
                    },
                    "example": "1769925929940537538"
                  }
                ],
                "responses": {
                  "200": {
                    "description": "Successful response",
                    "content": {
                      "application/json": {
                        "schema": {
                          "type": "object",
                          "properties": {
                            "imageURL": {
                              "type": "string"
                            },
                            "description": {
                              "type": "string"
                            },
                            "computed_cost": {
                              "type": "string"
                            }
                          }
                        }
                      }
                    }
                  },
                  "400": {
                    "description": "Bad request",
                    "content": {
                      "application/json": {
                        "schema": {
                          "type": "object",
                          "properties": {
                            "error": {
                              "type": "string"
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            },
            "/api/tweet-contract-call": {
              "get": {
                "summary": "Request a Transaction Object for to Submit Mint Tweet Intent",
                "description": "Generate a transaction object that the user needs to sign in order to mint a tweet. This transaction includes details such as the tweet ID, image URL, notification account, and the computed cost for the reward.",
                "operationId": "reserve-mint-transaction",
                "tags": [
                    "tweet",
                    "tweet Id",
                    "Produce",
                    "reserve-mint-transaction",
                    "generate-transaction"
                ],
                "parameters": [
                  {
                    "in": "query",
                    "name": "image_url",
                    "required": true,
                    "schema": {
                      "type": "string"
                    },
                    "description": "Image URL to be rewarded. This could be an IPFS URL in the format ipfs://{CID} or an Arweave URL in the format ar://{Image_ID}."
                  },
                  {
                    "in": "query",
                    "name": "tweet_id",
                    "required": true,
                    "schema": {
                      "type": "string"
                    },
                    "description": "The tweet ID of the post to be rewarded (Note: It is a 19-digit long ID)"
                  },
                  {
                    "in": "query",
                    "name": "notify",
                    "required": false,
                    "schema": {
                      "type": "string"
                    },
                    "example": "@ryan_soury",
                    "description": "The X (Twitter) account handle to notify when the reward/post is complete"
                  },
                  {
                    "in": "query",
                    "name": "computed_cost",
                    "required": true,
                    "schema": {
                      "type": "string"
                    },
                    "example": "680000000000000000000",
                    "description": "The required deposit amount for minting the tweet"
                  }
                ],
                "responses": {
                  "200": {
                    "description": "Successfully generated the transaction object for minting the tweet.",
                    "content": {
                      "application/json": {
                        "schema": {
                            "type": "object",
                            "properties": {
                              "receiverId": {
                                "type": "string",
                                "description": "The NEAR account ID of the contract that will receive the transaction. ie. CONTRACT_ID"
                              },
                              "functionCalls": {
                                "type": "array",
                                "items": {
                                  "type": "object",
                                  "properties": {
                                    "methodName": {
                                      "type": "string",
                                      "description": "The method name to be invoked on the contract."
                                    },
                                    "args": {
                                      "type": "object",
                                      "description": "The arguments required for the function call.",
                                      "properties": {
                                        "tweet_id": {
                                          "type": "string"
                                        },
                                        "image_url": {
                                          "type": "string"
                                        }
                                      },
                                      "additionalProperties": true
                                    },
                                    "gas": {
                                      "type": "string",
                                      "description": "The amount of gas to attach to the transaction, specified in yoctoNEAR."
                                    },
                                    "amount": {
                                      "type": "string",
                                      "description": "REQUIRED: The amount of NEAR tokens to attach to the transaction, specified in yoctoNEAR."
                                    }
                                  },
                                  "required": [
                                    "methodName",
                                    "args",
                                    "gas",
                                    "amount"
                                  ]
                                }
                              }
                            },
                            "required": [
                              "receiverId",
                              "functionCalls"
                            ]
                          }
                        }
                    }
                  }
                }
              }
            },
              "/api/tweet-cancel-call": {
              "get": {
                "summary": "Cancel an Initiatised Mint Intent",
                "description": "Cancel a previously initiated tweet intent",
                "operationId": "cancel-mint-intent",
                "tags": [
                  "tweet",
                  "contract",
                  "cancel"
                ],
                "parameters": [
                  {
                    "in": "query",
                    "name": "tweet_id",
                    "required": true,
                    "schema": {
                      "type": "string"
                    },
                    "description": "The ID of the tweet for which to cancel the contract call"
                  }
                ],
                "responses": {
                  "200": {
                    "description": "Successfully generated the transaction object for cancelling the minting the tweet.",
                    "content": {
                      "application/json": {
                        "schema": {
                            "type": "object",
                            "properties": {
                              "receiverId": {
                                "type": "string",
                                "description": "The NEAR account ID of the contract that will receive the transaction. ie. CONTRACT_ID"
                              },
                              "functionCalls": {
                                "type": "array",
                                "items": {
                                  "type": "object",
                                  "properties": {
                                    "methodName": {
                                      "type": "string",
                                      "description": "The method name to be invoked on the contract."
                                    },
                                    "args": {
                                      "type": "object",
                                      "description": "The arguments required for the function call.",
                                      "properties": {
                                        "tweet_id": {
                                          "type": "string"
                                        },
                                      },
                                      "additionalProperties": true
                                    },
                                    "gas": {
                                      "type": "string",
                                      "description": "The amount of gas to attach to the transaction, specified in yoctoNEAR."
                                    },
                                  },
                                  "required": [
                                    "methodName",
                                    "args",
                                    "gas",
                                    "amount"
                                  ]
                                }
                              }
                            },
                            "required": [
                              "receiverId",
                              "functionCalls"
                            ]
                          }
                      }
                    }
                  },
                  "400": {
                    "description": "Invalid request",
                    "content": {
                      "application/json": {
                        "schema": {
                          "type": "object",
                          "properties": {
                            "error": {
                              "type": "string",
                              "description": "Invalid request error"
                            }
                          }
                        }
                      }
                    }
                  },
                  "404": {
                    "description": "Tweet not found",
                    "content": {
                      "application/json": {
                        "schema": {
                          "type": "object",
                          "properties": {
                            "error": {
                              "type": "string",
                              "description": "X Post/Tweet not found error"
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
    );
    Json(json!({
            "openapi": "3.0.0",
            "info": {
              "title": "X NFTs: Minting & Management API",
              "description": "API for minting unique 1-of-1 NFTs from X (Twitter) posts, including capturing post snapshots, managing intents, and canceling intents.",
              "version": "1.0.0"
            },
            "servers": [
              {
                "url": plugin_url
              }
            ],
            "x-mb": {
              "account-id": account_id ,
              "assistant": {
                "name": "X NFTs - Assistant",
                "description": "An AI assistant designed to facilitate the minting of 1-of-1 NFTs from X (Twitter) posts, including capturing snapshots, managing intents, and handling cancellations.",
                "instructions": "When asked \"what can you help me with?\", introduce yourself and ask the User to provide the X (Twitter) Post URL. \n
                Step 1: Obtain the X (Twitter) post URL from the user's input. \n
                Step 2: Inquire if the user wishes to generate NFT art using Bitte AI or capture a snapshot of the X Post/Tweet (tweet-snapshot). \n
                Step 3: Upon user confirmation, display the image and request their X (Twitter) profile handle for notification purposes post-minting. \n
                Verify the user's profile and inform them that minting will proceed once the zkProof of the X (Twitter) Post is validated on the Near Blockchain. \n
                Guide the user to submit their transaction to initiate the process and ensure them that their profile will be notified upon completion.",
                "tools": [
                  {
                    "type": "generate-image"
                  },
                  {
                    "type": "generate-transaction"
                  }
                ]
              }
            },
            "paths": routes
          }
    ))
}
