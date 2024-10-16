use rocket::serde::json::{ self, json, Json, Value };
use std::{ fs, env, path::PathBuf };

use crate::handler::PluginInfo;

/// Route handler for serving the OpenAPI specification
#[get("/ai-plugin.json")]
pub async fn open_api_specification() -> Json<Value> {
    // Create a JSON object representing the OpenAPI specification
    let routes =
        json!({
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
                "summary": "Get tweet post data",
                "description": "This endpoint returns an image of the post to be Rewarded along side an tweet description",
                "operationId": "tweet-snapshot",
                "parameters": [
                  {
                    "name": "tweet_id",
                    "in": "query",
                    "description": "The tweet Id of the post to be Rewarded (NB: Its a 19 digit long numeric ID)",
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
                "summary": "Reward Transaction Request data/ create transaction",
                "description": "An array of transaction objects to be signed by user to Reward Request/Reward reserve",
                "operationId": "transaction",
                "tags": [
                    "tweet",
                    "tweet Id",
                    "Produce",
                    "transaction"
                ],
                "parameters": [
                  {
                    "in": "query",
                    "name": "image_url",
                    "required": true,
                    "schema": {
                      "type": "string"
                    },
                    "description": "Image URL to be Rewarded, this could be the IPFS or arweave URL in from ipfs://{CID} or ar://{Image_ID}"
                  },
                  {
                    "in": "query",
                    "name": "tweet_id",
                    "required": true,
                    "schema": {
                      "type": "string"
                    },
                    "description": "The tweet Id of the post to be Rewarded (NB: Its a 19 digit long ID)"
                  },
                  {
                    "in": "query",
                    "name": "notify",
                    "required": false,
                    "schema": {
                      "type": "string"
                    },
                    "example": "@ryan_soury",
                    "description": "The tweet account to notify when is reward/post is complete"
                  }
                ],
                "responses": {
                  "200": {
                    "description": "Reward transactions generated successfully.",
                    "content": {
                      "application/json": {
                        "schema": {
                          "type": "array",
                          "items": {
                            "type": "object",
                            "properties": {
                              "receiverId": {
                                "type": "string",
                                "description": "The account ID of the contract that will receive the transaction. CONTRACT_ID"
                              },
                              "functionCalls": {
                                "type": "array",
                                "items": {
                                  "type": "object",
                                  "properties": {
                                    "methodName": {
                                      "type": "string",
                                      "description": "The name of the method to be called on the contract."
                                    },
                                    "args": {
                                      "type": "object",
                                      "description": "Arguments for the function call.",
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
                                      "description": "The amount of gas to attach to the transaction, in yoctoNEAR."
                                    },
                                    "amount": {
                                      "type": "string",
                                      "description": "REQUIRED: The amount of NEAR tokens to attach to the transaction, in yoctoNEAR."
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
              }
            }
          }
    );
    Json(
        json!({
            "openapi": "3.0.0",
            "info": {
              "title": "Tweet post rewarder",
              "description": "API for retrieving a digital representation of a post image along with it's description for further image generation if necessary and clones a tweet",
              "version": "1.0.0"
            },
            "servers": [
              {
                "url": env::var("HOST_URL").unwrap_or_else(|_| {
                    let current_dir = env::current_dir().unwrap();
                    let mut bitte_config_path = PathBuf::from(current_dir);
                    bitte_config_path.push("bitte.dev.json");
                    let bitte_config = fs::read_to_string(bitte_config_path).unwrap();
                    let plugin_info:PluginInfo  = json::serde_json::from_str(bitte_config.as_str()).unwrap();
                    plugin_info.url
                  }
                )
              }
            ],
            "x-mb": {
              "account-id": env::var("ACCOUNT_ID").unwrap_or(String::from("<missing>.near")),
              "assistant": {
                "name": "Post Cloner",
                "description": "An assistant that provides a digital representation of a Post as an Image with its description and generates a custom transaction for the user",
                "instructions": "Retrieve the X(twitter) post URL from the user's request. Ask the user if they want to AI generated art for the post or use the default tweet-snapshot that will be provided. If the user confirms, Show the Image and prompt them to provide the user profile to notify after minting. Confirm the user's profile and inform them that the post will be minted once verified on the Near Blockchain. Instruct the user to submit their transaction to get started and assure them that the specified profile will be notified once it's ready.",
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
    )
    )
}
